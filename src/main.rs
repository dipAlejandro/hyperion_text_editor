mod buffer;
mod cli;
mod config;
mod editor;
mod search;
mod syntax;
mod tabs;
mod terminal;
mod ui;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use std::io::Write;

use crate::{
    cli::Args,
    editor::Editor,
    tabs::Tabs,
    terminal::{clear_screen, keys, messages, request_input},
};

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
                        editor.click_at(mouse_event.column, mouse_event.row);
                    }
                    MouseEventKind::ScrollUp => editor.scroll_up(3),
                    MouseEventKind::ScrollDown => editor.scroll_down(3),
                    _ => {}
                }
                editor.adjust_scroll();
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
                    let path = request_input(&mut stdout, "Abrir archivo: ");
                    if !path.is_empty() {
                        tabs.current_mut().open_file(&path);
                    } else {
                        tabs.current_mut().state_msg = messages::OPEN_CANCELLED.to_string();
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
}
