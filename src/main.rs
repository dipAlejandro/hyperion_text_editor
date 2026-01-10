use std::{
    fs,
    io::{self, Write},
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    filename: Option<String>,
    state_msg: String,
    window_sizes: (u16, u16),
    offset_row: usize,
    offset_col: usize,
}

impl Editor {
    fn new() -> Self {
        let window_sizes = termion::terminal_size().unwrap_or((80, 24));

        Editor {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            filename: None,
            state_msg: String::from("Ctrl+Q: Salir | Ctrl+S: Guardar | Ctrl+O: Abrir"),
            window_sizes,
            offset_row: 0,
            offset_col: 0,
        }
    }

    fn open_file(&mut self, path: &str) {
        // Intentar leer el archivo completo como un string
        match fs::read_to_string(path) {
            Ok(content) => {
                // Si se logra, dividir el contenido en lineas
                // El collect() convierte el iterador en un Vec<String>
                self.lines = content.lines().map(|line| line.to_string()).collect();

                // Si el archivo esta vacio asegurar tener al menos una linea
                if self.lines.is_empty() {
                    self.lines.push(String::new());
                }

                // Guardar el nombre del archivo y resetear el cursor
                self.filename = Some(path.to_string());
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.offset_row = 0;
                self.offset_col = 0;
                self.state_msg = format!("Archivo '{}' cargado correctamente", path);
            }
            Err(e) => {
                self.state_msg = format!("Error al abrir el archivo: {}", e);
            }
        }
    }

    fn save_file(&mut self, path: &str) {
        let content = self.lines.join("\n");

        //Intentar escribir el contenido del archivo
        match fs::write(path, content) {
            Ok(_) => {
                self.filename = Some(path.to_string());
                self.state_msg = format!("Archivo '{}' guardado correctamente.", path);
            }
            Err(e) => {
                self.state_msg = format!("Error al intentar guardar el archivo: {}", e);
            }
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
            self.cursor_y += 1;

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

    fn adjust_scroll(&mut self) {
        // Calcular cuantas lineas se pueden mostrar (dejar 2 para barra de estado)
        let visible_lines = (self.window_sizes.1 - 2) as usize;

        // Si el cursor está por encima de la ventana visible, ajustamos hacia arriba
        if self.cursor_y < self.offset_row {
            self.offset_row = self.cursor_y;
        }

        // Si el cursor está por debajo de la ventana visible, ajustar hacia abajo
        // Restar 1 porque las posiciones empiezan en 0
        if self.cursor_y >= self.offset_row + visible_lines {
            self.offset_row = self.cursor_y - visible_lines + 1;
        }

        // Calcular el ancho del area de numeros de linea
        // Necesitamos saber cuantos digitos tiene el numero de linea mas alto
        let line_num_digits = self.lines.len().to_string().len();
        let line_num_width = line_num_digits + 2;

        // Calcular cuantas columnas tenemos disponibles para el texto
        let visible_cols = (self.window_sizes.0 as usize).saturating_sub(line_num_width);

        // Ajustar scroll horizontal si es necesario
        if self.cursor_x < self.offset_col {
            self.offset_col = self.cursor_x;
        }

        if self.cursor_x >= self.offset_col + visible_cols {
            self.offset_col = self.cursor_x - visible_cols + 1;
        }
    }

    fn write<W: Write>(&self, stdout: &mut W) {
        write!(stdout, "{}", termion::clear::All).unwrap();

        // Calcular cuantas lineas se pueden mostrar (dejar 2 para barra de estado)
        let visible_lines = (self.window_sizes.1 - 2) as usize;

        // Calcular ancho necesario para los numero de linea
        // Esto depende de cuantas lineas tiene el documento en total
        let line_num_digits = self.lines.len().to_string().len();
        let line_num_width = line_num_digits + 2;

        // Iterar solo sobre las lineas que estan actualmente visibles
        // self.offset_row nos dice cual es la primera linea visible
        let start = self.offset_row;
        let end = (self.offset_row + visible_lines).min(self.lines.len());

        for i in start..end {
            let line_num = i + 1;
            let window_row = (i - self.offset_row + 1) as u16;

            // Dibujar el número de línea con un color diferente
            // El formato {:>width$} alinea el número a la derecha
            write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(1, window_row),
                termion::color::Fg(termion::color::Cyan),
                format!("{:>width$} ", line_num, width = line_num_digits),
                termion::color::Fg(termion::color::Reset)
            )
            .unwrap();

            // Obtenemos la porcion visible  de la linea considerando el scroll horizontal
            let line = &self.lines[i];
            let start_col = self.offset_col.min(line.len());
            let visible_line = &line[start_col..];

            write!(stdout, "{}", visible_line).unwrap();
        }

        // Dibujar la barra de estado en la parte inferior
        let state_row = self.window_sizes.1 - 1;

        // Crear la información de la barra de estado
        let info_file = match &self.filename {
            Some(name) => name.clone(),
            None => String::from("[Sin nombre]"),
        };

        let info_position = format!(
            "Linea {}/{}, Col {}",
            self.cursor_y + 1,
            self.lines.len(),
            self.cursor_x + 1
        );

        // Dibujar la barra de estado con fondo invertido
        write!(
            stdout,
            "{}{}{}{}",
            termion::cursor::Goto(1, state_row),
            termion::style::Invert,
            format!("{} | {}", info_file, info_position),
            termion::style::Reset
        )
        .unwrap();

        // Limpiar cualquier texto que pudiera quedar de la linea de estado anterior
        write!(stdout, "{}", termion::clear::AfterCursor).unwrap();

        // Dibujar la barra de estado en la ultima linea
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, state_row + 1),
            &self.state_msg
        )
        .unwrap();

        // Calcular la posicion visual del cursor en la pantalla
        // Tenemos que considerar el offset de scroll y el ancho de los numeros de linea
        let visual_cursor_x =
            (self.cursor_x.saturating_sub(self.offset_col) + line_num_width) as u16;
        let visual_cursor_y = (self.cursor_y.saturating_sub(self.offset_row) + 1) as u16;

        write!(
            stdout,
            "{}",
            termion::cursor::Goto(visual_cursor_x, visual_cursor_y)
        )
        .unwrap();

        stdout.flush().unwrap();
    }
}

fn request_entry<W: Write>(stdout: &mut W, prompt: &str) -> String {
    let stdin = io::stdin();
    let mut user_in = String::new();

    // Mostrar prompt en la ultima linea
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

    // Leer caracter por caracter hasta que el usuario presione Enter
    for k in stdin.keys() {
        match k.unwrap() {
            Key::Char('\n') => break,
            Key::Char(c) => {
                user_in.push(c);
                write!(stdout, "{}", c).unwrap();
                stdout.flush().unwrap();
            }

            Key::Backspace => {
                if !user_in.is_empty() {
                    user_in.pop();
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
    user_in
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
        // Limpiar el mensaje de estado antes de procesar la siguiente tecla
        // Esto hace que los mensajes temporales desaparezcan después de cualquier acción

        if !editor.state_msg.starts_with("Ctrl+") {
            editor.state_msg = String::from("Ctrl+Q: Salir | Ctrl+S: Guardar | Ctrl+O: Abrir");
        }
        match k.unwrap() {
            //Ctrl + Q para salir
            Key::Ctrl('q') => break,

            // Ctrl+S para guardar
            Key::Ctrl('s') => {
                let path = match &editor.filename {
                    Some(name) => name.clone(),
                    None => {
                        // Si no hay nombre de archivo pedimos uno
                        let name = request_entry(&mut stdout, "Guardar como: ");
                        if name.is_empty() {
                            editor.state_msg = String::from("Guardado cancelado");
                            editor.write(&mut stdout);
                            continue;
                        }
                        name
                    }
                };
                editor.save_file(&path);
            }

            // Ctrl+O para abrir archivo
            Key::Ctrl('o') => {
                let path = request_entry(&mut stdout, "Abrir archivo: ");
                if !path.is_empty() {
                    editor.open_file(&path);
                } else {
                    editor.state_msg = String::from("Apertura cancelada");
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

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
}
