use std::io::Write;

use crate::{buffer::TextBuffer, search::SearchState, terminal::messages, ui};

pub struct Editor {
    pub buffer: TextBuffer,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub filename: Option<String>,
    pub state_msg: String,
    pub window_sizes: (u16, u16),
    pub offset_row: usize,
    pub offset_col: usize,
    pub search: SearchState,
}

impl Editor {
    pub fn new() -> Self {
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

    pub fn open_file(&mut self, path: &str) {
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

    pub fn save_file(&mut self, path: &str) {
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

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(self.cursor_y, self.cursor_x, c);

        self.cursor_x += 1;
    }

    pub fn new_line(&mut self) {
        let (new_y, new_x) = self.buffer.split_line(self.cursor_y, self.cursor_x);
        self.cursor_y = new_y;
        self.cursor_x = new_x;
    }

    pub fn delete_char(&mut self) {
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

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            // Ajustamos cursor_x si la nueva línea es más corta
            self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y < self.buffer.line_count() - 1 {
            self.cursor_y += 1;

            // Ajustamos cursor_x si la nueva línea es más corta
            self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Si estamos al inicio de una línea, vamos al final de la línea anterior
            self.cursor_y -= 1;
            self.cursor_x = self.buffer.line_length(self.cursor_y);
        }
    }

    pub fn move_right(&mut self) {
        let line_length = self.buffer.line_length(self.cursor_y);
        if self.cursor_x < line_length {
            self.cursor_x += 1;
        } else if self.cursor_y < self.buffer.line_count() - 1 {
            // Si estamos al final de una línea, vamos al inicio de la siguiente
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn adjust_scroll(&mut self) {
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

    pub fn search(&mut self, query: &str) {
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

    pub fn jump_to_current_match(&mut self) {
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

    pub fn next_match(&mut self) {
        if self.search.next_match().is_some() {
            self.jump_to_current_match();
        } else {
            self.state_msg = messages::NO_ACTIVE_SEARCH.to_string();
        }
    }

    pub fn previous_match(&mut self) {
        if self.search.previous_match().is_some() {
            self.jump_to_current_match();
        } else {
            self.state_msg = messages::NO_ACTIVE_SEARCH.to_string();
        }
    }

    /**
     * coord: (y, x)
     * */
    pub fn go_to_line(&mut self, coords: (usize, usize)) {
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

    pub fn write<W: Write>(&self, stdout: &mut W) {
        ui::clear_screen(stdout);

        // Calcular cuantas lineas se pueden mostrar (dejar 2 para barra de estado)
        let visible_lines = (self.window_sizes.1 - 2) as usize;
        let line_num_width = ui::calculate_line_number_width(self.buffer.line_count());

        let start = self.offset_row;
        let end = (self.offset_row + visible_lines).min(self.buffer.line_count());

        // Renderizar cada linea visible
        for i in start..end {
            let line_num = i + 1;
            let window_row = (i - self.offset_row + 1) as u16;
            let line_num_digits = self.buffer.line_count().to_string().len();

            // Dibujar numero de linea
            ui::render_line_number(stdout, line_num, window_row, line_num_digits);

            // Dibujar contenido de la línea con resaltado de búsqueda si aplica
            let line = &self.buffer.lines()[i];
            ui::render_line_content(stdout, line, i, self.offset_col, &self.search);
        }

        // Dibujar barra de estado
        let state_row = self.window_sizes.1 - 1;
        ui::render_status_bar(
            stdout,
            state_row,
            self.filename.as_deref(),
            self.cursor_y + 1,
            self.buffer.line_count(),
            self.cursor_x + 1,
        );

        // Renderizar mensaje de estado
        ui::render_message(stdout, state_row + 1, &self.state_msg);

        // Posicionar el cursor
        let (visual_x, visual_y) = ui::calculate_visual_cursor_position(
            self.cursor_x,
            self.cursor_y,
            self.offset_col,
            self.offset_row,
            line_num_width,
        );
        ui::position_cursor(stdout, visual_x, visual_y);

        stdout.flush().unwrap();
    }
}
