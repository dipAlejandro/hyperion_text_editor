use std::io::{self, Write};

use termion::{event::Key, input::TermRead, raw::IntoRawMode, raw::RawTerminal};

// Tipo alias para simplificar la firma de funciones que escriben a stdout
pub type StdoutRaw = RawTerminal<io::Stdout>;

// Constantes para declarar teclas de control
pub mod keys {
    use termion::event::Key;

    pub const QUIT: Key = Key::Ctrl('q');
    pub const SAVE: Key = Key::Ctrl('s');
    pub const OPEN: Key = Key::Ctrl('o');
    pub const SEARCH: Key = Key::Ctrl('f');
    pub const NEXT_MATCH: Key = Key::Ctrl('n');
    pub const PREV_MATCH: Key = Key::Ctrl('p');
    pub const GOTO_LINE: Key = Key::Ctrl('g');
}

// Constates para manejar el estado por defecto
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

pub fn init_raw_mode() -> io::Result<StdoutRaw> {
    io::stdout().into_raw_mode()
}

pub fn read_keys() -> termion::input::Keys<io::Stdin> {
    io::stdin().keys()
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
    let stdin = io::stdin();

    let mut user_input = String::new();

    // Obtener la altura de la terminal,
    let (_, height) = termion::terminal_size().unwrap_or((80, 24));

    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Goto(1, height),
        termion::clear::CurrentLine,
        prompt
    )
    .unwrap();

    stdout.flush().unwrap();

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('\n') => break,
            Key::Char(c) => {
                user_input.push(c);
                write!(stdout, "{}", c).unwrap();
                stdout.flush().unwrap();
            }

            Key::Backspace => {
                if !user_input.is_empty() {
                    user_input.pop();
                    write!(
                        stdout,
                        "{} {}",
                        termion::cursor::Left(1),
                        termion::cursor::Left(1)
                    )
                    .unwrap();
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
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();

    stdout.flush().unwrap();
}
