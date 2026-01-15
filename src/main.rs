mod buffer;
mod cli;
mod editor;
mod search;
mod terminal;
mod ui;

use crossterm::event::KeyCode;
use std::io::Write;

use crate::{
    cli::Args,
    editor::Editor,
    terminal::{clear_screen, keys, messages, request_input},
};

fn main() {
    let args = Args::parse_args();

    let mut stdout = terminal::init_raw_mode().unwrap();

    let mut editor = Editor::new();

    // Si se proporcionó un archivo, intentar abrirlo o preparar para crearlo
    if let Some(filepath) = args.file {
        if std::path::Path::new(&filepath).exists() {
            editor.open_file(&filepath);
        } else {
            editor.filename = Some(filepath.clone());
            editor.state_msg = format!("Nuevo archivo: '{}' (Ctrl+S para guardar)", filepath);
        }
    } else {
        clear_screen(&mut stdout);
        write!(stdout, "Editor de Texto - Presiona 'q' para salir \r\n\r\n").unwrap();
        stdout.flush().unwrap();
    }

    editor.write(&mut stdout);

    // Leer entrada de usuario
    loop {
        let key = match terminal::read_key() {
            Ok(k) => k,
            Err(_) => break,
        };

        // Limpiar el mensaje de estado antes de procesar la siguiente tecla
        if !editor.state_msg.starts_with("Ctrl+")
            && !editor.state_msg.starts_with("Nuevo archivo:")
            && !editor.state_msg.starts_with("Archivo '")
            && !editor.state_msg.starts_with("Encontradas")
            && !editor.state_msg.starts_with("Coincidencia")
            && !editor.state_msg.starts_with("Posicionado")
        {
            editor.state_msg = messages::DEFAULT_STATUS.to_string();
        }

        if keys::is_quit(&key) {
            break;
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
            match key.code {
                KeyCode::Up => editor.move_up(),
                KeyCode::Down => editor.move_down(),
                KeyCode::Left => editor.move_left(),
                KeyCode::Right => editor.move_right(),
                KeyCode::Enter => editor.new_line(),
                KeyCode::Backspace => editor.delete_char(),
                KeyCode::Char(c) => editor.insert_char(c),
                _ => {}
            }
        }

        editor.adjust_scroll();
        editor.write(&mut stdout);
    }

    clear_screen(&mut stdout);
    terminal::cleanup().unwrap();
}
