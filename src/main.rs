use std::io::{self, Read, Write};

use termion::raw::IntoRawMode;

fn main() {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();

    write!(
        stdout,
        "{}{}Editor de Texto - Presiona 'q' para salir \r\n\r\n",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();

    stdout.flush().unwrap();

    for byte in stdin.bytes() {
        let byte = byte.unwrap();
        let character = byte as char;

        if character == 'q' {
            break;
        }

        write!(stdout, "{}", character).unwrap();
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::clear::All).unwrap();
}
