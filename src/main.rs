mod buffer;
mod editor;
mod search;
mod terminal;
mod ui;

use std::io::Write;
use termion::event::Key;

use crate::{
    editor::Editor,
    terminal::{clear_screen, keys, messages, request_input},
};

fn main() {
    let mut stdout = terminal::init_raw_mode().unwrap();

    let mut editor = Editor::new();

    editor.write(&mut stdout);

    write!(
        stdout,
        "{}{}Editor de Texto - Presiona 'q' para salir \r\n\r\n",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();

    stdout.flush().unwrap();

    // Leer entrada de usuario
    for k in terminal::read_keys() {
        // Limpiar el mensaje de estado antes de procesar la siguiente tecla
        // Esto hace que los mensajes temporales desaparezcan después de cualquier acción

        if !editor.state_msg.starts_with("Ctrl+") {
            editor.state_msg = messages::DEFAULT_STATUS.to_string();
        }
        match k.unwrap() {
            keys::QUIT => break,

            keys::SAVE => {
                let path = match &editor.filename {
                    Some(name) => name.clone(),
                    None => {
                        // Si no hay nombre de archivo pedimos uno
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
            }

            keys::OPEN => {
                let path = request_input(&mut stdout, "Abrir archivo: ");
                if !path.is_empty() {
                    editor.open_file(&path);
                } else {
                    editor.state_msg = messages::OPEN_CANCELLED.to_string();
                }
            }

            keys::SEARCH => {
                let query = request_input(&mut stdout, "Buscar: ");
                editor.search(&query);
            }

            keys::NEXT_MATCH => editor.next_match(),
            keys::PREV_MATCH => editor.previous_match(),

            keys::GOTO_LINE => {
                let coords_str = request_input(&mut stdout, "Ir a (linea, columna): ");

                // Intentar parsear las coordenadas de manera segura
                let parts: Vec<&str> = coords_str.split(',').collect();

                // Verificar que tenemos exactamente dos partes
                if parts.len() != 2 {
                    editor.state_msg = messages::INVALID_FORMAT.to_string();
                    editor.write(&mut stdout);
                    continue;
                }

                // Intentar parsear cada parte como número
                match (
                    parts[0].trim().parse::<usize>(),
                    parts[1].trim().parse::<usize>(),
                ) {
                    (Ok(line), Ok(col)) => {
                        // Verificar que los números no sean cero (el usuario ingresa base-1)
                        if line == 0 || col == 0 {
                            editor.state_msg = messages::LINES_START_AT_ONE.to_string();
                        } else {
                            // Convertir de base-1 (usuario) a base-0 (interno)
                            editor.go_to_line((line - 1, col - 1));
                        }
                    }
                    _ => {
                        editor.state_msg = messages::INVALID_NUMBERS.to_string();
                    }
                }
            }

            //Teclas de navegación
            Key::Up => editor.move_up(),
            Key::Down => editor.move_down(),
            Key::Left => editor.move_left(),
            Key::Right => editor.move_right(),

            // Enter crea nueva linea
            Key::Char('\n') => editor.new_line(),

            // Backspace para borrar caracteres
            Key::Backspace => editor.delete_char(),

            // Cualquier otro caracter se inserta
            Key::Char(c) => editor.insert_char(c),

            _ => {}
        }

        editor.adjust_scroll();
        editor.write(&mut stdout);
    }
    clear_screen(&mut stdout);
}
