mod buffer;
mod cli;
mod config;
mod editor;
mod picker;
mod search;
mod syntax;
mod tabs;
mod terminal;
mod ui;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use std::io::Write;
use std::path::Path;

use crate::{
    cli::Args,
    editor::Editor,
    picker::FilePicker,
    tabs::Tabs,
    terminal::{clear_screen, keys, messages, request_input},
};

fn run_file_picker<W: Write>(stdout: &mut W) -> Option<String> {
    let root = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let mut picker = FilePicker::open(&root);

    loop {
        let (_, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let visible_rows = (height as usize).saturating_sub(1);
        picker.adjust_scroll(visible_rows);

        let current_dir_display = picker.current_dir().to_string_lossy().into_owned();

        ui::render_picker(
            stdout,
            ui::PickerViewport {
                current_dir: &current_dir_display,
                query: picker.query(),
                entries: picker.entries(),
                selected: picker.selected(),
                scroll_offset: picker.scroll_offset(),
            },
        );
        stdout.flush().unwrap();

        match terminal::read_event() {
            Ok(Event::Key(key)) => match key.code {
                KeyCode::Esc => return None,
                KeyCode::Enter => {
                    if let Some(path) = picker.activate() {
                        return Some(path.to_string_lossy().into_owned());
                    }
                    // era un directorio: activate() ya navegó, seguimos el loop
                }
                KeyCode::Up => picker.move_up(),
                KeyCode::Down => picker.move_down(),
                KeyCode::Backspace => picker.backspace(),
                KeyCode::Char(c) => picker.push_char(c),
                _ => {}
            },
            Ok(_) => {}
            Err(_) => return None,
        }
    }
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::cleanup();
        default_hook(info);
    }));
}
/// Intenta cerrar la pestaña activa, pidiendo confirmación si tiene
/// cambios sin guardar. Devuelve true si el programa debe salir
/// (se cerró la última pestaña).
fn try_close_tab(tabs: &mut Tabs) -> bool {
    if !tabs.current_mut().confirm_quit() {
        return false;
    }
    tabs.close_current()
}
fn render(tabs: &Tabs, stdout: &mut impl Write) {
    tabs.current().write(stdout);
    let width = tabs.current().width();
    let theme = tabs.current().ui_theme();
    ui::render_tab_bar(
        stdout,
        width,
        &tabs.labels(),
        theme.tab_bar_bg,
        theme.tab_bar_fg,
    );
    let (x, y) = tabs.current().cursor_screen_position();
    ui::position_cursor(stdout, x, y);
    stdout.flush().unwrap();
}

/// Despacha las teclas de movimiento, selección y portapapeles que no
/// requieren entrada interactiva adicional (sin request_input).
/// Devuelve true si la tecla fue manejada acá.
fn dispatch_non_interactive_key(editor: &mut Editor, key: &KeyEvent) -> bool {
    if keys::is_undo(key) {
        editor.undo();
    } else if keys::is_redo(key) {
        editor.redo();
    } else if keys::is_select_all(key) {
        editor.select_all();
    } else if keys::is_cut(key) {
        editor.cut_selection();
    } else if keys::is_copy_line(key) {
        editor.copy_line();
    } else if keys::is_copy_selection(key) {
        editor.copy_selection();
    } else if keys::is_paste(key) {
        editor.paste_clipboard();
    } else {
        match key.code {
            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                editor.start_or_clear_selection();
                editor.move_up();
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                editor.start_or_clear_selection();
                editor.move_down();
            }
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
                editor.start_or_clear_selection();
                editor.move_left();
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
                editor.start_or_clear_selection();
                editor.move_right();
            }
            KeyCode::Up => {
                editor.clear_selection();
                editor.move_up();
            }
            KeyCode::Down => {
                editor.clear_selection();
                editor.move_down();
            }
            KeyCode::Left => {
                editor.clear_selection();
                editor.move_left();
            }
            KeyCode::Right => {
                editor.clear_selection();
                editor.move_right();
            }
            KeyCode::Home => {
                editor.clear_selection();
                editor.move_to_line_start();
            }
            KeyCode::End => {
                editor.clear_selection();
                editor.move_to_line_end();
            }
            KeyCode::PageUp => editor.move_page_up(),
            KeyCode::PageDown => editor.move_page_down(),
            KeyCode::Tab => editor.insert_tab(),
            KeyCode::Enter => editor.new_line(),
            KeyCode::Backspace => editor.delete_char(),
            KeyCode::Delete => editor.delete_forward_char(),
            KeyCode::Char(c) => editor.insert_char(c),
            _ => return false,
        }
    }
    true
}

fn main() {
    install_panic_hook();
    let args = Args::parse_args();

    let mut stdout = terminal::init_raw_mode().unwrap();

    let mut tabs = Tabs::new();

    if let Some(filepath) = args.file {
        let editor = tabs.current_mut();
        if std::path::Path::new(&filepath).exists() {
            editor.open_file(&filepath);
        } else {
            editor.filename = Some(filepath.clone());
            editor.state_msg = format!("Nuevo archivo: '{}' (Ctrl+S para guardar)", filepath);
        }
    } else {
        clear_screen(&mut stdout);
        write!(
            stdout,
            "Editor de Texto - Presiona 'Ctrl + q' para salir \r\n\r\n"
        )
        .unwrap();
        stdout.flush().unwrap();
    }

    render(&tabs, &mut stdout);

    while let Ok(event) = terminal::read_event() {
        match event {
            Event::Resize(width, height) => {
                tabs.current_mut().update_window_size(width, height);
                tabs.current_mut().adjust_scroll();
                render(&tabs, &mut stdout);
                continue;
            }
            Event::Mouse(mouse_event) => {
                let editor = tabs.current_mut();
                match mouse_event.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        editor.start_selection_at(mouse_event.column, mouse_event.row);
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        editor.extend_selection_to(mouse_event.column, mouse_event.row);
                    }
                    MouseEventKind::ScrollUp => editor.scroll_up(3),
                    MouseEventKind::ScrollDown => editor.scroll_down(3),
                    _ => {}
                }
                editor.adjust_scroll();
                editor.write(&mut stdout);
                continue;
            }
            Event::Paste(text) => {
                tabs.current_mut().insert_text(&text);
                tabs.current_mut().adjust_scroll();
                render(&tabs, &mut stdout);
                continue;
            }
            Event::Key(key) => {
                if !keys::is_quit(&key) {
                    tabs.current_mut().reset_pending_quit();
                }

                {
                    let editor = tabs.current_mut();
                    if !editor.state_msg.starts_with(messages::DEFAULT_STATUS)
                        && !editor.state_msg.starts_with("Nuevo archivo:")
                        && !editor.state_msg.starts_with("Archivo '")
                        && !editor.state_msg.starts_with("Encontradas")
                        && !editor.state_msg.starts_with("Coincidencia")
                        && !editor.state_msg.starts_with("Posicionado")
                    {
                        editor.state_msg = messages::DEFAULT_STATUS.to_string();
                    }
                }

                if keys::is_quit(&key) {
                    if tabs.current_mut().confirm_quit() {
                        if try_close_tab(&mut tabs) {
                            break;
                        }
                    }
                    render(&tabs, &mut stdout);
                    continue;
                } else if keys::is_new_tab(&key) {
                    tabs.new_tab();
                } else if keys::is_close_tab(&key) {
                    if try_close_tab(&mut tabs) {
                        break;
                    }
                } else if keys::is_next_tab(&key) {
                    tabs.next_tab();
                } else if keys::is_previous_tab(&key) {
                    tabs.previous_tab();
                } else if keys::is_save(&key) {
                    let path = match &tabs.current().filename {
                        Some(name) => name.clone(),
                        None => {
                            let name = request_input(&mut stdout, "Guardar como: ");
                            if name.is_empty() {
                                tabs.current_mut().state_msg = messages::SAVE_CANCELLED.to_string();
                                render(&tabs, &mut stdout);
                                continue;
                            }
                            name
                        }
                    };
                    tabs.current_mut().save_file(&path);
                } else if keys::is_open(&key) {
                    match run_file_picker(&mut stdout) {
                        Some(path) => tabs.current_mut().open_file(&path),
                        None => tabs.current_mut().state_msg = messages::OPEN_CANCELLED.to_string(),
                    }
                } else if keys::is_search(&key) {
                    let query = request_input(&mut stdout, "Buscar: ");
                    tabs.current_mut().search(&query);
                } else if keys::is_replace_current(&key) {
                    if !tabs.current().has_replacement() {
                        let replacement = request_input(&mut stdout, "Reemplazar con: ");
                        tabs.current_mut().set_replacement(&replacement);
                    }
                    tabs.current_mut().replace_current_match();
                } else if keys::is_replace_all(&key) {
                    let replacement = request_input(&mut stdout, "Reemplazar todo con: ");
                    tabs.current_mut().set_replacement(&replacement);
                    tabs.current_mut().replace_all_matches();
                } else if keys::is_next_match(&key) {
                    tabs.current_mut().next_match();
                } else if keys::is_prev_match(&key) {
                    tabs.current_mut().previous_match();
                } else if keys::is_goto_line(&key) {
                    let coords_str = request_input(&mut stdout, "Ir a (linea, columna): ");
                    let parts: Vec<&str> = coords_str.split(',').collect();

                    if parts.len() != 2 {
                        tabs.current_mut().state_msg = messages::INVALID_FORMAT.to_string();
                        render(&tabs, &mut stdout);
                        continue;
                    }

                    match (
                        parts[0].trim().parse::<usize>(),
                        parts[1].trim().parse::<usize>(),
                    ) {
                        (Ok(line), Ok(col)) => {
                            if line == 0 || col == 0 {
                                tabs.current_mut().state_msg =
                                    messages::LINES_START_AT_ONE.to_string();
                            } else {
                                tabs.current_mut().go_to_line((line - 1, col - 1));
                            }
                        }
                        _ => {
                            tabs.current_mut().state_msg = messages::INVALID_NUMBERS.to_string();
                        }
                    }
                } else {
                    dispatch_non_interactive_key(tabs.current_mut(), &key);
                }

                tabs.current_mut().adjust_scroll();
                render(&tabs, &mut stdout);
            }
            _ => {}
        }
    }
    clear_screen(&mut stdout);
    terminal::cleanup().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn typing_characters_inserts_them() {
        let mut editor = Editor::new();

        for c in ['h', 'o', 'l', 'a'] {
            dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char(c), KeyModifiers::NONE));
        }

        assert_eq!(editor.line_content(0), "hola");
    }
    #[test]
    fn shift_arrow_selects_and_ctrl_c_copies_selection() {
        let mut editor = Editor::new();
        for c in ['h', 'o', 'l', 'a'] {
            dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char(c), KeyModifiers::NONE));
        }

        // Volver al inicio de la línea y seleccionar con Shift+Right x2.
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Home, KeyModifiers::NONE));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Right, KeyModifiers::SHIFT));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Right, KeyModifiers::SHIFT));

        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('c'), KeyModifiers::CONTROL));

        // Mover el cursor al final y pegar para verificar qué se copió.
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::End, KeyModifiers::NONE));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('v'), KeyModifiers::CONTROL));

        assert_eq!(editor.line_content(0), "holaho");
    }

    #[test]
    fn ctrl_z_undoes_last_insertion() {
        let mut editor = Editor::new();
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('a'), KeyModifiers::NONE));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('b'), KeyModifiers::NONE));

        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('z'), KeyModifiers::CONTROL));

        assert_eq!(editor.line_content(0), "a");
    }

    #[test]
    fn ctrl_a_selects_all_and_ctrl_x_cuts() {
        let mut editor = Editor::new();
        for c in ['h', 'i'] {
            dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char(c), KeyModifiers::NONE));
        }

        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('a'), KeyModifiers::CONTROL));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('x'), KeyModifiers::CONTROL));

        assert_eq!(editor.line_content(0), "");
    }

    #[test]
    fn enter_preserves_indentation() {
        let mut editor = Editor::new();
        for c in [' ', ' ', 'x'] {
            dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char(c), KeyModifiers::NONE));
        }

        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Enter, KeyModifiers::NONE));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('y'), KeyModifiers::NONE));

        assert_eq!(editor.line_content(1), "  y");
    }
    #[cfg(test)]
    mod tests {
        use crate::tabs::Tabs;

        #[test]
        fn new_tab_and_switch_between_tabs() {
            let mut tabs = Tabs::new();
            assert_eq!(tabs.labels().len(), 1);

            tabs.new_tab();
            assert_eq!(tabs.labels().len(), 2);

            tabs.current_mut().insert_char('a');
            assert_eq!(tabs.current().filename, None);

            tabs.previous_tab();
            tabs.current_mut().insert_char('b');

            tabs.next_tab();
            // Volvimos a la segunda pestaña, que sigue teniendo 'a'
            assert!(tabs.current().is_dirty());
        }

        #[test]
        fn close_tab_removes_it_and_keeps_others_intact() {
            let mut tabs = Tabs::new();
            tabs.new_tab();
            tabs.new_tab();
            assert_eq!(tabs.labels().len(), 3);

            let closed_last = tabs.close_current();
            assert!(!closed_last);
            assert_eq!(tabs.labels().len(), 2);
        }

        #[test]
        fn closing_last_tab_signals_quit() {
            let mut tabs = Tabs::new();
            let closed_last = tabs.close_current();
            assert!(closed_last);
        }

        #[test]
        fn each_tab_has_independent_dirty_state() {
            let mut tabs = Tabs::new();
            tabs.current_mut().insert_char('x');
            assert!(tabs.current().is_dirty());

            tabs.new_tab();
            assert!(!tabs.current().is_dirty());

            tabs.previous_tab();
            assert!(tabs.current().is_dirty());
        }

        #[test]
        fn mouse_click_moves_cursor_independently_per_tab() {
            let mut tabs = Tabs::new();
            tabs.current_mut().update_window_size(80, 24);
            tabs.current_mut().insert_char('a');
            tabs.current_mut().insert_char('b');
            tabs.current_mut().insert_char('c');

            tabs.new_tab();
            tabs.current_mut().update_window_size(80, 24);
            tabs.current_mut().insert_char('x');

            // Click en la pestaña 2 en columna 0 no debe mover el cursor de la pestaña 1
            tabs.current_mut().click_at(0, 1);

            tabs.previous_tab();
            assert_eq!(tabs.current().cursor_position(), (3, 0));
        }

        #[test]
        fn save_and_open_round_trip_through_tabs() {
            let path = std::env::temp_dir().join(format!(
                "hyperion_main_test_{}_{}.txt",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let path_str = path.to_str().unwrap().to_string();

            let mut tabs = Tabs::new();
            tabs.current_mut().insert_char('h');
            tabs.current_mut().insert_char('i');
            tabs.current_mut().save_file(&path_str);
            assert!(!tabs.current().is_dirty());

            tabs.new_tab();
            tabs.current_mut().open_file(&path_str);
            assert_eq!(tabs.current().filename.as_deref(), Some(path_str.as_str()));

            let _ = std::fs::remove_file(&path);
        }
    }
    #[test]
    fn ctrl_shift_v_pastes_like_ctrl_v() {
        let mut editor = Editor::new();
        for c in ['h', 'i'] {
            dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char(c), KeyModifiers::NONE));
        }
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('a'), KeyModifiers::CONTROL));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        dispatch_non_interactive_key(&mut editor, &key(KeyCode::End, KeyModifiers::NONE));

        dispatch_non_interactive_key(
            &mut editor,
            &key(
                KeyCode::Char('V'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
        );

        assert_eq!(editor.line_content(0), "hihi");
    }
    #[test]
    fn insert_text_does_not_compound_indentation() {
        let mut editor = Editor::new();
        editor.insert_text("fn main() {\n    let x = 1;\n}\n");

        assert_eq!(editor.line_content(0), "fn main() {");
        assert_eq!(editor.line_content(1), "    let x = 1;");
        assert_eq!(editor.line_content(2), "}");
    }
    #[test]
    fn click_at_top_content_row_selects_first_visible_line() {
        let mut editor = Editor::new();
        editor.update_window_size(80, 24);
        editor.insert_text("primera\nsegunda\ntercera");

        // Fila de pantalla 1 es la primera fila de contenido (fila 0 es la tab bar).
        editor.click_at(0, 1);

        assert_eq!(editor.cursor_position(), (0, 0));
    }

    #[test]
    fn drag_selection_matches_visual_rows() {
        let mut editor = Editor::new();
        editor.update_window_size(80, 24);
        editor.insert_text("primera\nsegunda\ntercera");
        editor.click_at(0, 1);

        editor.start_selection_at(0, 1);
        editor.extend_selection_to(0, 3); // arrastra hasta "tercera"

        assert_eq!(editor.cursor_position(), (0, 2));
    }
}
