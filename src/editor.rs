//! Módulo principal del editor de texto
//!
//! Contiene la estructura `Editor` que coordina todas las operaciones
//! del editor incluyendo navegación, edición, búsqueda y renderizado.

use std::io::Write;

use crossterm::{cursor, terminal};

use crate::{buffer::TextBuffer, search::SearchState, terminal::messages, ui};

pub struct Editor {
    buffer: TextBuffer,
    cursor_x: usize,
    cursor_y: usize,
    pub filename: Option<String>,
    pub state_msg: String,
    window_sizes: (u16, u16),
    offset_row: usize,
    offset_col: usize,
    search: SearchState,
    clipboard: String,
}

impl Editor {
    pub fn new() -> Self {
        let window_sizes = terminal::size().unwrap_or((80, 24));

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
            clipboard: String::new(),
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
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let prev_len = self.buffer.join_with_previous(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = prev_len;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y < self.buffer.line_count() - 1 {
            self.cursor_y += 1;
            self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.buffer.line_length(self.cursor_y);
        }
    }

    pub fn move_right(&mut self) {
        let line_length = self.buffer.line_length(self.cursor_y);
        if self.cursor_x < line_length {
            self.cursor_x += 1;
        } else if self.cursor_y < self.buffer.line_count() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn adjust_scroll(&mut self) {
        let visible_lines = (self.window_sizes.1 - 2) as usize;

        if self.cursor_y < self.offset_row {
            self.offset_row = self.cursor_y;
        }

        if self.cursor_y >= self.offset_row + visible_lines {
            self.offset_row = self.cursor_y - visible_lines + 1;
        }

        let line_num_digits = self.buffer.line_count().to_string().len();
        let line_num_width = line_num_digits + 2;
        let visible_cols = (self.window_sizes.0 as usize).saturating_sub(line_num_width);

        if self.cursor_x < self.offset_col {
            self.offset_col = self.cursor_x;
        }

        if visible_cols == 0 {
            self.offset_col = 0;
            return;
        }

        if self.cursor_x >= self.offset_col + visible_cols {
            self.offset_col = self.cursor_x - visible_cols + 1;
        }
    }

    pub fn update_window_size(&mut self, width: u16, height: u16) {
        self.window_sizes = (width, height);

        let visible_lines = height.saturating_sub(2) as usize;
        let line_count = self.buffer.line_count();
        let max_visible_lines = visible_lines.max(1);
        let max_offset_row = line_count.saturating_sub(max_visible_lines);

        if self.offset_row > self.cursor_y {
            self.offset_row = self.cursor_y;
        }
        if self.offset_row > max_offset_row {
            self.offset_row = max_offset_row;
        }

        let line_num_width = ui::calculate_line_number_width(self.buffer.line_count());
        let visible_cols = width
            .saturating_sub(line_num_width as u16)
            .max(1) as usize;
        let line_length = self.buffer.line_length(self.cursor_y);
        let max_offset_col = line_length.saturating_sub(visible_cols);

        if self.offset_col > self.cursor_x {
            self.offset_col = self.cursor_x;
        }
        if self.offset_col > max_offset_col {
            self.offset_col = max_offset_col;
        }
    }

    pub fn search(&mut self, query: &str) {
        let lines: Vec<String> = self.buffer.iter_lines().collect();
        let count = self.search.search(query, &lines);

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

    pub fn go_to_line(&mut self, coords: (usize, usize)) {
        if !self.buffer.is_valid_line(coords.0) {
            self.state_msg = format!(
                "Línea {} no existe. El documento tiene {} líneas",
                coords.0 + 1,
                self.buffer.line_count()
            );
            return;
        }

        self.cursor_y = coords.0;
        let line_length = self.buffer.line_length(self.cursor_y);

        if coords.1 >= line_length {
            self.cursor_x = line_length;
            self.state_msg = format!(
                "Columna {} fuera de rango. Posicionado al final de la línea (columna {})",
                coords.1 + 1,
                line_length
            );
        } else {
            self.cursor_x = coords.1;
            self.state_msg = format!(
                "Posicionado en línea {}, columna {}",
                self.cursor_y + 1,
                self.cursor_x + 1
            );
        }
    }

    pub fn copy_line(&mut self) {
        self.clipboard = self.buffer.line(self.cursor_y);
        if self.clipboard.is_empty() {
            self.state_msg = "Línea vacía copiada".to_string();
        } else {
            self.state_msg = "Línea copiada".to_string();
        }
    }

    pub fn paste_clipboard(&mut self) {
        if self.clipboard.is_empty() {
            self.state_msg = "Portapapeles vacío".to_string();
            return;
        }

        let lines: Vec<&str> = self.clipboard.split('\n').collect();
        self.buffer
            .insert_str(self.cursor_y, self.cursor_x, &self.clipboard);

        if lines.len() == 1 {
            self.cursor_x += lines[0].chars().count();
        } else {
            self.cursor_y += lines.len() - 1;
            self.cursor_x = lines.last().unwrap_or(&"").chars().count();
        }
    }

    pub fn write<W: Write>(&self, stdout: &mut W) {
        let mut out: Vec<u8> = Vec::with_capacity(16 * 1024);

        write!(out, "{}", cursor::Hide).unwrap();
        write!(out, "{}", cursor::MoveTo(0, 0)).unwrap();
        write!(
            out,
            "{}",
            terminal::Clear(terminal::ClearType::FromCursorDown)
        )
        .unwrap();

        let visible_lines = self.window_sizes.1.saturating_sub(2) as usize;

        if visible_lines == 0 || self.window_sizes.0 == 0 {
            ui::render_message(&mut out, 0, "Ventana demasiado pequeña");
            write!(out, "{}", cursor::Show).unwrap();
            stdout.write_all(&out).unwrap();
            stdout.flush().unwrap();
            return;
        }
        let line_num_width = ui::calculate_line_number_width(self.buffer.line_count());

        let start = self.offset_row;
        let end = (self.offset_row + visible_lines).min(self.buffer.line_count());

        for i in start..end {
            let line_num = i + 1;
            let window_row = (i - self.offset_row) as u16;
            let _line_num_digits = self.buffer.line_count().to_string().len();

            ui::render_line_number(&mut out, line_num, window_row, line_num_width); // valor
            // anterior:
            // line_num_digits
            let line = self.buffer.line(i);
            ui::render_line_content(
                &mut out,
                &line,
                i,
                self.offset_col,
                &self.search,
                i == self.cursor_y,
            );
        }

        let state_row = self.window_sizes.1 - 2;
        ui::render_status_bar(
            &mut out,
            state_row,
            self.filename.as_deref(),
            self.cursor_y + 1,
            self.buffer.line_count(),
            self.cursor_x + 1,
        );

        ui::render_message(&mut out, state_row + 1, &self.state_msg);

        let (visual_x, visual_y) = ui::calculate_visual_cursor_position(
            self.cursor_x,
            self.cursor_y,
            self.offset_col,
            self.offset_row,
            line_num_width,
        );
        ui::position_cursor(&mut out, visual_x, visual_y);

        write!(out, "{}", cursor::Show).unwrap();

        stdout.write_all(&out).unwrap();
        stdout.flush().unwrap();
    }
}
