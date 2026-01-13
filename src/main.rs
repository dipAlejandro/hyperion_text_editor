mod buffer;
mod search;
mod terminal;

use std::io::Write;
use termion::event::Key;

use crate::{
    buffer::TextBuffer,
    search::SearchState,
    terminal::{clear_screen, keys, messages, request_input},
};

struct Editor {
    buffer: TextBuffer,
    cursor_x: usize,
    cursor_y: usize,
    filename: Option<String>,
    state_msg: String,
    window_sizes: (u16, u16),
    offset_row: usize,
    offset_col: usize,
    search: SearchState,
}

impl Editor {
    fn new() -> Self {
        let window_sizes = termion::terminal_size().unwrap_or((80, 24));

        Editor {
            buffer: TextBuffer::new(),
            cursor_x: 0,
            cursor_y: 0,
            filename: None,
            state_msg: messages::DEFAULT_STATUS.to_string(),
            window_sizes,
            offset_row: 0,
            offset_col: 0,
            search: SearchState::new(),
        }
    }

    fn open_file(&mut self, path: &str) {
        match TextBuffer::from_file(path) {
            Ok(buffer) => {
                self.buffer = buffer;
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
        //Intentar escribir el contenido del archivo
        match self.buffer.save_to_file(path) {
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
        self.buffer.insert_char(self.cursor_y, self.cursor_x, c);

        self.cursor_x += 1;
    }

    fn new_line(&mut self) {
        let (new_y, new_x) = self.buffer.split_line(self.cursor_y, self.cursor_x);
        self.cursor_y = new_y;
        self.cursor_x = new_x;
    }

    fn delete_char(&mut self) {
        if self.buffer.delete_char(self.cursor_y, self.cursor_x) {
            // Se eliminó un carácter en la misma línea
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Necesitamos unir con la línea anterior
            let prev_len = self.buffer.join_with_previous(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = prev_len;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            // Ajustamos cursor_x si la nueva línea es más corta
            self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y < self.buffer.line_count() - 1 {
            self.cursor_y += 1;

            // Ajustamos cursor_x si la nueva línea es más corta
            self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Si estamos al inicio de una línea, vamos al final de la línea anterior
            self.cursor_y -= 1;
            self.cursor_x = self.buffer.line_length(self.cursor_y);
        }
    }

    fn move_right(&mut self) {
        let line_length = self.buffer.line_length(self.cursor_y);
        if self.cursor_x < line_length {
            self.cursor_x += 1;
        } else if self.cursor_y < self.buffer.line_count() - 1 {
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
        let line_num_digits = self.buffer.line_count().to_string().len();
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

    fn search(&mut self, query: &str) {
        let count = self.search.search(query, self.buffer.lines());

        if query.is_empty() {
            self.state_msg = messages::SEARCH_CANCELLED.to_string();
            return;
        }

        if count > 0 {
            self.jump_to_current_match();
            self.state_msg = format!("Encontradas {} coincidencias de '{}'", count, query);
        } else {
            self.state_msg = format!("No se encontró '{}'", query);
        }
    }

    fn jump_to_current_match(&mut self) {
        if let Some(m) = self.search.current_match() {
            self.cursor_y = m.line;
            self.cursor_x = m.start_col;
            if let Some(idx) = self.search.current_index() {
                self.state_msg = format!(
                    "Coincidencia {}/{}: '{}'",
                    idx + 1,
                    self.search.match_count(),
                    self.search.query().unwrap_or(&String::new())
                );
            }
        }
    }

    fn next_match(&mut self) {
        if self.search.next_match().is_some() {
            self.jump_to_current_match();
        } else {
            self.state_msg = messages::NO_ACTIVE_SEARCH.to_string();
        }
    }

    fn previous_match(&mut self) {
        if self.search.previous_match().is_some() {
            self.jump_to_current_match();
        } else {
            self.state_msg = messages::NO_ACTIVE_SEARCH.to_string();
        }
    }

    /**
     * coord: (y, x)
     * */
    fn go_to_line(&mut self, coords: (usize, usize)) {
        // Validar que la línea existe
        // Recordar que coords.0 es base-1 (como lo ve el usuario)
        // pero internamente trabajamos con base-0
        if !self.buffer.is_valid_line(coords.0) {
            // La línea no existe, mostrar error y no mover el cursor
            self.state_msg = format!(
                "Línea {} no existe. El documento tiene {} líneas",
                coords.0 + 1,
                self.buffer.line_count()
            );
            return;
        }

        // La línea es válida, mover el cursor
        self.cursor_y = coords.0;

        // Ahora validar la columna basándonos en la línea donde acabamos de posicionar el cursor
        let line_length = self.buffer.line_length(self.cursor_y);

        if coords.1 >= line_length {
            // Si la columna está fuera de rango, ir al final de la línea
            self.cursor_x = line_length;
            self.state_msg = format!(
                "Columna {} fuera de rango. Posicionado al final de la línea (columna {})",
                coords.1 + 1,
                line_length
            );
        } else {
            // La columna es válida
            self.cursor_x = coords.1;
            self.state_msg = format!(
                "Posicionado en línea {}, columna {}",
                self.cursor_y + 1,
                self.cursor_x + 1
            );
        }
    }

    fn write<W: Write>(&self, stdout: &mut W) {
        write!(stdout, "{}", termion::clear::All).unwrap();

        // Calcular cuantas lineas se pueden mostrar (dejar 2 para barra de estado)
        let visible_lines = (self.window_sizes.1 - 2) as usize;

        // Calcular ancho necesario para los numero de linea
        // Esto depende de cuantas lineas tiene el documento en total
        let line_num_digits = self.buffer.line_count().to_string().len();
        let line_num_width = line_num_digits + 2;

        // Iterar solo sobre las lineas que estan actualmente visibles
        // self.offset_row nos dice cual es la primera linea visible
        let start = self.offset_row;
        let end = (self.offset_row + visible_lines).min(self.buffer.line_count());

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
            let line = &self.buffer.lines()[i];
            let start_col = self.offset_col.min(line.len());
            let visible_line = &line[start_col..];

            // Si hay una busqueda activa resaltar las coincidencias
            if self.search.is_active() {
                let mut current_pos = 0;
                let mut highlighted_line = String::new();

                // Buscar todas las coincidencias en esta linea que esten visibles
                for m in self.search.matches() {
                    if m.line == i && m.start_col >= start_col {
                        // Añadir texto normal antes de la coincidencia
                        let text_before_len = m
                            .start_col
                            .saturating_sub(start_col)
                            .saturating_sub(current_pos);

                        if text_before_len > 0 && current_pos < visible_line.len() {
                            let end_idx = (current_pos + text_before_len).min(visible_line.len());
                            highlighted_line.push_str(&visible_line[current_pos..end_idx]);
                            current_pos = end_idx;
                        }

                        // Añadir coincidencia resaltada
                        let match_start = m.start_col.saturating_sub(start_col);
                        let match_end = m.end_col.saturating_sub(start_col).min(visible_line.len());

                        if match_start < visible_line.len() && match_end > match_start {
                            highlighted_line.push_str(&format!(
                                "{}{}{}",
                                termion::color::Bg(termion::color::Yellow),
                                &visible_line[match_start..match_end],
                                termion::color::Bg(termion::color::Reset)
                            ));
                            current_pos = match_end;
                        }
                    }
                }
                // Añadir el resto de la linea después de todas las coincidencia
                if current_pos < visible_line.len() {
                    highlighted_line.push_str(&visible_line[current_pos..]);
                }

                write!(stdout, "{}", highlighted_line).unwrap();
            } else {
                write!(stdout, "{}", visible_line).unwrap();
            }
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
            self.buffer.line_count(),
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
