mod buffer;
mod cli;
mod config;
mod editor;
mod search;
mod syntax;
mod terminal;
mod ui;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind,
};
use std::io::Write;

use crate::{
    cli::Args,
    editor::Editor,
    terminal::{clear_screen, keys, messages, request_input},
};

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::cleanup();
        default_hook(info);
    }));
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

    let mut editor = Editor::new();

    if let Some(filepath) = args.file {
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

    editor.write(&mut stdout);

    while let Ok(event) = terminal::read_event() {
        match event {
            Event::Resize(width, height) => {
                editor.update_window_size(width, height);
                editor.adjust_scroll();
                editor.write(&mut stdout);
                continue;
            }
            Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        editor.click_at(mouse_event.column, mouse_event.row);
                    }
                    MouseEventKind::ScrollUp => editor.scroll_up(3),
                    MouseEventKind::ScrollDown => editor.scroll_down(3),
                    _ => {}
                }
                editor.adjust_scroll();
                editor.write(&mut stdout);
                continue;
            }
            Event::Key(key) => {
                if !keys::is_quit(&key) {
                    editor.reset_pending_quit();
                }

                if !editor.state_msg.starts_with(messages::DEFAULT_STATUS)
                    && !editor.state_msg.starts_with("Nuevo archivo:")
                    && !editor.state_msg.starts_with("Archivo '")
                    && !editor.state_msg.starts_with("Encontradas")
                    && !editor.state_msg.starts_with("Coincidencia")
                    && !editor.state_msg.starts_with("Posicionado")
                {
                    editor.state_msg = messages::DEFAULT_STATUS.to_string();
                }

                if keys::is_quit(&key) {
                    if editor.confirm_quit() {
                        break;
                    }
                    editor.write(&mut stdout);
                    continue;
                } else if keys::is_save(&key) {
                    let path = match &editor.filename {
                        Some(name) => name.clone(),
                        None => {
                            let name = request_input(&mut stdout, "Guardar como: ");
                            if name.is_empty() {
                                editor.state_msg = messages::SAVE_CANCELLED.to_string();
                                editor.write(&mut stdout);
                                continue;
                            }
                            name
                        }
                    };
                    editor.save_file(&path);
                } else if keys::is_open(&key) {
                    let path = request_input(&mut stdout, "Abrir archivo: ");
                    if !path.is_empty() {
                        editor.open_file(&path);
                    } else {
                        editor.state_msg = messages::OPEN_CANCELLED.to_string();
                    }
                } else if keys::is_search(&key) {
                    let query = request_input(&mut stdout, "Buscar: ");
                    editor.search(&query);
                } else if keys::is_replace_current(&key) {
                    if !editor.has_replacement() {
                        let replacement = request_input(&mut stdout, "Reemplazar con: ");
                        editor.set_replacement(&replacement);
                    }
                    editor.replace_current_match();
                } else if keys::is_replace_all(&key) {
                    let replacement = request_input(&mut stdout, "Reemplazar todo con: ");
                    editor.set_replacement(&replacement);
                    editor.replace_all_matches();
                } else if keys::is_next_match(&key) {
                    editor.next_match();
                } else if keys::is_prev_match(&key) {
                    editor.previous_match();
                } else if keys::is_goto_line(&key) {
                    let coords_str = request_input(&mut stdout, "Ir a (linea, columna): ");
                    let parts: Vec<&str> = coords_str.split(',').collect();

                    if parts.len() != 2 {
                        editor.state_msg = messages::INVALID_FORMAT.to_string();
                        editor.write(&mut stdout);
                        continue;
                    }

                    match (
                        parts[0].trim().parse::<usize>(),
                        parts[1].trim().parse::<usize>(),
                    ) {
                        (Ok(line), Ok(col)) => {
                            if line == 0 || col == 0 {
                                editor.state_msg = messages::LINES_START_AT_ONE.to_string();
                            } else {
                                editor.go_to_line((line - 1, col - 1));
                            }
                        }
                        _ => {
                            editor.state_msg = messages::INVALID_NUMBERS.to_string();
                        }
                    }
                } else {
                    dispatch_non_interactive_key(&mut editor, &key);
                }

                editor.adjust_scroll();
                editor.write(&mut stdout);
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
