/// Terminal.rs
use std::io::{self, Write};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{self, ClearType},
};

// Constantes para declarar teclas de control
pub mod keys {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    pub fn is_quit(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('q')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_save(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('s')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_open(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('o')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_search(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('f')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_next_match(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('n')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_prev_match(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('p')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_goto_line(key: &KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('g')) && key.modifiers.contains(KeyModifiers::CONTROL)
    }
}

// Constantes para manejar el estado por defecto
pub mod messages {
    pub const DEFAULT_STATUS: &str = "Ctrl+Q: Salir | Ctrl+S: Guardar | Ctrl+O: Abrir";
    pub const SAVE_CANCELLED: &str = "Guardado cancelado";
    pub const OPEN_CANCELLED: &str = "Apertura cancelada";
    pub const SEARCH_CANCELLED: &str = "Búsqueda cancelada";
    pub const NO_ACTIVE_SEARCH: &str = "No hay búsqueda activa";
    pub const INVALID_FORMAT: &str = "Formato inválido. Use: linea,columna";
    pub const INVALID_NUMBERS: &str = "Ingrese números válidos";
    pub const LINES_START_AT_ONE: &str = "Las líneas y columnas empiezan en 1";
}

pub fn init_raw_mode() -> io::Result<io::Stdout> {
    terminal::enable_raw_mode()?;
    Ok(io::stdout())
}

pub fn cleanup() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    io::stdout().execute(cursor::Show)?;
    Ok(())
}

/// Lee el siguiente evento de teclado
pub fn read_key() -> io::Result<KeyEvent> {
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(key);
            }
        }
    }
}

/// Solicita entrada del usuario con un prompt
///
/// # Argumentos
/// * `stdout` - Terminal en modo raw donde escribir
/// * `prompt` - Mensaje a mostrar al usuario
///
/// # Retorna
/// El texto ingresado por el usuario
pub fn request_input<W: Write>(stdout: &mut W, prompt: &str) -> String {
    let mut user_input = String::new();

    // Obtener la altura de la terminal
    let (_, height) = terminal::size().unwrap_or((80, 24));

    stdout
        .queue(cursor::MoveTo(0, height - 1))
        .unwrap()
        .queue(terminal::Clear(ClearType::CurrentLine))
        .unwrap();
    write!(stdout, "{}", prompt).unwrap();
    stdout.flush().unwrap();

    while let Ok(key) = read_key() {
        match key.code {
            KeyCode::Enter => break,
            KeyCode::Char(c) => {
                user_input.push(c);
                write!(stdout, "{}", c).unwrap();
                stdout.flush().unwrap();
            }

            KeyCode::Backspace => {
                if !user_input.is_empty() {
                    user_input.pop();
                    stdout.queue(cursor::MoveLeft(1)).unwrap();
                    write!(stdout, " ").unwrap();
                    stdout.queue(cursor::MoveLeft(1)).unwrap();
                    stdout.flush().unwrap();
                }
            }
            _ => {}
        }
    }

    user_input
}

/// Limpiar pantalla y resetear cursor
pub fn clear_screen<W: Write>(stdout: &mut W) {
    stdout
        .queue(terminal::Clear(ClearType::All))
        .unwrap()
        .queue(cursor::MoveTo(0, 0))
        .unwrap();
    stdout.flush().unwrap();
}
