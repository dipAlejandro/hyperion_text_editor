use std::io::{self, Read, Write};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
}

impl Editor {
    fn new() -> Self {
        Editor {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn insert_char(&mut self, c: char) {
        let current_line = &mut self.lines[self.cursor_y];
        current_line.insert(self.cursor_x, c);

        self.cursor_x += 1;
    }

    fn new_line(&mut self) {
        let current_line = &self.lines[self.cursor_y];
        let right_txt = current_line[self.cursor_x..].to_string();

        self.lines[self.cursor_y].truncate(self.cursor_x);

        self.lines.insert(self.cursor_y + 1, right_txt);

        self.cursor_x = 0;
        self.cursor_y += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_x > 0 {
            // Si no estamos al inicio de la línea, borramos el carácter anterior
            self.lines[self.cursor_y].remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Si estamos al inicio de la línea pero no en la primera línea
            // juntamos esta línea con la anterior
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
            self.lines[self.cursor_y].push_str(&current_line);
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            // Ajustamos cursor_x si la nueva línea es más corta
            let line_length = self.lines[self.cursor_y].len();
            if self.cursor_x > line_length {
                self.cursor_x = line_length;
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y -= 1;

            // Ajustamos cursor_x si la nueva línea es más corta
            let line_length = self.lines[self.cursor_y].len();
            if self.cursor_x > line_length {
                self.cursor_x = line_length;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Si estamos al inicio de una línea, vamos al final de la línea anterior
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
    }

    fn move_right(&mut self) {
        let line_length = self.lines[self.cursor_y].len();
        if self.cursor_x < line_length {
            self.cursor_x += 1;
        } else if self.cursor_y < self.lines.len() - 1 {
            // Si estamos al final de una línea, vamos al inicio de la siguiente
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn write<W: Write>(&self, stdout: &mut W) {
        write!(stdout, "{}", termion::clear::All).unwrap();

        for (i, line) in self.lines.iter().enumerate() {
            // Posicionamos el cursor al inicio de cada línea (columna 1, fila i+1)
            write!(
                stdout,
                "{}{}",
                termion::cursor::Goto(1, (i + 1) as u16),
                line
            )
            .unwrap();
        }
        // Posicionamos el cursor en la ubicación correcta
        // Sumamos 1 porque las coordenadas de la terminal empiezan en 1, no en 0
        write!(
            stdout,
            "{}",
            termion::cursor::Goto((self.cursor_x + 1) as u16, (self.cursor_y + 1) as u16)
        )
        .unwrap();

        stdout.flush().unwrap();
    }
}

fn main() {
    // Activar modo raw en la terminal
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();

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
    for k in stdin.keys() {
        match k.unwrap() {
            //Ctrl + Q para salir
            Key::Ctrl('q') => break,

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

        editor.write(&mut stdout);
    }

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
}
