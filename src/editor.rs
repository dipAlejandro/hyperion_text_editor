//! Módulo principal del editor de texto
//!
//! Contiene la estructura `Editor` que coordina todas las operaciones
//! del editor incluyendo navegación, edición, búsqueda y renderizado.

use std::io::Write;

use crossterm::{cursor, terminal};
use ropey::Rope;

use crate::{
    buffer::TextBuffer,
    config::{
        load_editor_config, load_language_config, load_syntax_theme, load_ui_theme, LanguageConfig,
        SyntaxTheme, UiTheme,
    },
    search::SearchState,
    terminal::messages,
    ui,
};

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
    selection_anchor: Option<(usize, usize)>,
    replacement: String,
    syntax_theme: SyntaxTheme,
    ui_theme: UiTheme,
    tab_size: usize,
    language_config: LanguageConfig,
    show_line_numbers: bool,
    undo_stack: Vec<UndoState>,
    redo_stack: Vec<UndoState>,
    dirty: bool,
    pending_quit: bool,
}

struct UndoState {
    rope: Rope,
    cursor_x: usize,
    cursor_y: usize,
}

impl Editor {
    const MAX_UNDO_HISTORY: usize = 200;
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
            syntax_theme: load_syntax_theme(),
            ui_theme: load_ui_theme(),
            tab_size: load_editor_config().tab_size,
            language_config: load_language_config(),
            show_line_numbers: load_editor_config().show_line_numbers,
            replacement: String::new(),
            selection_anchor: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
            pending_quit: false,
        }
    }
    pub fn width(&self) -> usize {
        self.window_sizes.0 as usize
    }

    pub fn cursor_screen_position(&self) -> (u16, u16) {
        let line_num_width = self.line_num_width();
        let (x, y) = ui::calculate_visual_cursor_position(
            self.cursor_x,
            self.cursor_y,
            self.offset_col,
            self.offset_row,
            line_num_width,
        );
        (x, y + 1)
    }
    fn line_num_width(&self) -> usize {
        if self.show_line_numbers {
            ui::calculate_line_number_width(self.buffer.line_count())
        } else {
            0
        }
    }
    fn push_undo_snapshot(&mut self) {
        self.undo_stack.push(UndoState {
            rope: self.buffer.snapshot(),
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
        });
        if self.undo_stack.len() > Self::MAX_UNDO_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.dirty = true
    }
    /// Inserta texto crudo (multilínea) sin pasar por la auto-indentación
    /// de `new_line`. Usado para pegado externo (bracketed paste) y clipboard interno.
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.delete_selection();
        self.push_undo_snapshot();

        let lines: Vec<&str> = text.split('\n').collect();
        self.buffer.insert_str(self.cursor_y, self.cursor_x, text);

        if lines.len() == 1 {
            self.cursor_x += lines[0].chars().count();
        } else {
            self.cursor_y += lines.len() - 1;
            self.cursor_x = lines.last().unwrap_or(&"").chars().count();
        }
        self.search.clear();
    }
    pub fn undo(&mut self) {
        let Some(state) = self.undo_stack.pop() else {
            self.state_msg = "Nada para deshacer".to_string();
            return;
        };

        self.redo_stack.push(UndoState {
            rope: self.buffer.snapshot(),
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
        });

        self.buffer.restore(state.rope);
        self.cursor_x = state.cursor_x;
        self.cursor_y = state.cursor_y;
        self.search.clear();
        self.selection_anchor = None;
        self.state_msg = "Deshecho".to_string();
    }

    pub fn redo(&mut self) {
        let Some(state) = self.redo_stack.pop() else {
            self.state_msg = "Nada para rehacer".to_string();
            return;
        };

        self.undo_stack.push(UndoState {
            rope: self.buffer.snapshot(),
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
        });

        self.buffer.restore(state.rope);
        self.cursor_x = state.cursor_x;
        self.cursor_y = state.cursor_y;
        self.search.clear();
        self.selection_anchor = None;
        self.state_msg = "Rehecho".to_string();
    }
    pub fn set_replacement(&mut self, replacement: &str) {
        self.replacement = replacement.to_string();
    }

    pub fn has_replacement(&self) -> bool {
        !self.replacement.is_empty()
    }

    pub fn replace_current_match(&mut self) {
        let Some(m) = self.search.current_match().cloned() else {
            self.state_msg = messages::NO_ACTIVE_SEARCH.to_string();
            return;
        };

        self.push_undo_snapshot();

        self.buffer.replace_range(
            (m.line, m.start_col),
            (m.line, m.end_col),
            &self.replacement,
        );

        // Recalcular matches ya que el texto cambió.
        let query = self.search.query().cloned().unwrap_or_default();
        let lines: Vec<String> = self.buffer.iter_lines().collect();
        let count = self.search.search(&query, &lines);

        self.cursor_y = m.line;
        self.cursor_x = m.start_col + self.replacement.chars().count();

        if count > 0 {
            self.jump_to_current_match();
            self.state_msg = format!("Reemplazado. {} coincidencias restantes", count);
        } else {
            self.state_msg = "Reemplazado. Sin más coincidencias".to_string();
        }
    }
    pub fn replace_all_matches(&mut self) {
        let Some(query) = self.search.query().cloned() else {
            self.state_msg = messages::NO_ACTIVE_SEARCH.to_string();
            return;
        };
        let replacement = self.replacement.clone();

        let lines: Vec<String> = self.buffer.iter_lines().collect();
        let mut temp_search = SearchState::new();
        let count = temp_search.search(&query, &lines);

        if count == 0 {
            self.state_msg = format!("No se encontró '{}'", query);
            return;
        }

        self.push_undo_snapshot();

        // Reemplazar de atrás hacia adelante para no invalidar posiciones ya calculadas.
        for m in temp_search.matches().iter().rev() {
            self.buffer
                .replace_range((m.line, m.start_col), (m.line, m.end_col), &replacement);
        }

        self.cursor_y = 0;
        self.cursor_x = 0;
        self.selection_anchor = None;
        self.search.clear();
        self.state_msg = format!("{} reemplazos de '{}' por '{}'", count, query, replacement);
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
                self.dirty = false;
                self.pending_quit = false;
                self.state_msg = format!("Archivo '{}' cargado correctamente", path);
            }
            Err(e) => {
                self.state_msg = format!("Error al abrir el archivo: {}", e);
            }
        }
    }

    pub fn ui_theme(&self) -> UiTheme {
        self.ui_theme
    }

    pub fn save_file(&mut self, path: &str) {
        match self.buffer.save_to_file(path) {
            Ok(_) => {
                self.filename = Some(path.to_string());
                self.state_msg = format!("Archivo '{}' guardado correctamente.", path);
                self.dirty = false;
                self.pending_quit = false;
            }
            Err(e) => {
                self.state_msg = format!("Error al intentar guardar el archivo: {}", e);
            }
        }
    }
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marca que se pidió salir con cambios sin guardar. Devuelve true
    /// si ya se había pedido antes (confirmación) y por lo tanto se puede salir.
    pub fn confirm_quit(&mut self) -> bool {
        if !self.dirty {
            return true;
        }

        if self.pending_quit {
            return true;
        }

        self.pending_quit = true;
        self.state_msg =
            "Cambios sin guardar. Presioná Ctrl+Q de nuevo para salir sin guardar.".to_string();
        false
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        self.push_undo_snapshot();
        self.buffer.insert_char(self.cursor_y, self.cursor_x, c);
        self.cursor_x += 1;
        self.search.clear();
    }

    pub fn reset_pending_quit(&mut self) {
        self.pending_quit = false;
    }

    pub fn new_line(&mut self) {
        self.delete_selection();
        self.push_undo_snapshot();

        let indent = self.buffer.leading_whitespace(self.cursor_y);
        let (new_y, new_x) = self.buffer.split_line(self.cursor_y, self.cursor_x);
        self.cursor_y = new_y;
        self.cursor_x = new_x;

        if !indent.is_empty() {
            self.buffer
                .insert_str(self.cursor_y, self.cursor_x, &indent);
            self.cursor_x += indent.chars().count();
        }

        self.search.clear();
    }

    pub fn insert_tab(&mut self) {
        self.push_undo_snapshot();
        let spaces = " ".repeat(self.tab_size);
        self.buffer
            .insert_str(self.cursor_y, self.cursor_x, &spaces);
        self.cursor_x += spaces.chars().count();
        self.search.clear();
    }

    pub fn delete_char(&mut self) {
        if self.delete_selection() {
            return;
        }
        self.push_undo_snapshot();
        if self.buffer.delete_char(self.cursor_y, self.cursor_x) {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let prev_len = self.buffer.join_with_previous(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = prev_len;
        }
        self.search.clear();
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };

        self.push_undo_snapshot();
        self.buffer.delete_range(start, end);
        self.cursor_y = start.0;
        self.cursor_x = start.1;
        self.selection_anchor = None;
        self.search.clear();
        true
    }
    pub fn select_all(&mut self) {
        let last_line = self.buffer.line_count() - 1;
        let last_col = self.buffer.line_length(last_line);

        self.selection_anchor = Some((0, 0));
        self.cursor_y = last_line;
        self.cursor_x = last_col;
        self.state_msg = "Todo seleccionado".to_string();
    }
    pub fn cut_selection(&mut self) {
        let Some((start, end)) = self.selection_range() else {
            self.state_msg = "Nada seleccionado".to_string();
            return;
        };

        self.clipboard = self.extract_range(start, end);
        self.delete_selection();
        self.state_msg = "Selección cortada".to_string();
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

    pub fn move_to_line_start(&mut self) {
        self.cursor_x = 0;
    }

    pub fn move_to_line_end(&mut self) {
        self.cursor_x = self.buffer.line_length(self.cursor_y);
    }

    pub fn move_page_up(&mut self) {
        let page_size = self.window_sizes.1.saturating_sub(4).max(1) as usize;
        self.cursor_y = self.cursor_y.saturating_sub(page_size);
        self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
    }

    pub fn move_page_down(&mut self) {
        let page_size = self.window_sizes.1.saturating_sub(4).max(1) as usize;
        let max_row = self.buffer.line_count().saturating_sub(1);
        self.cursor_y = (self.cursor_y + page_size).min(max_row);
        self.cursor_x = self.buffer.clamp_column(self.cursor_y, self.cursor_x);
    }

    /// Convierte coordenadas de pantalla a una posición (línea, columna) del buffer.
    fn resolve_click_position(&self, screen_col: u16, screen_row: u16) -> Option<(usize, usize)> {
        let visible_lines = self.window_sizes.1.saturating_sub(4) as usize;

        // La fila 0 es la barra de pestañas; el contenido arranca en la fila 1.
        if screen_row == 0 {
            return None;
        }
        let clicked_row = (screen_row - 1) as usize;

        if visible_lines == 0 || clicked_row >= visible_lines {
            return None;
        }

        let last_line = self.buffer.line_count().saturating_sub(1);
        let target_y = (self.offset_row + clicked_row).min(last_line);

        let line_num_width = self.line_num_width();
        let clicked_col = screen_col as usize;
        let target_x = clicked_col
            .saturating_sub(line_num_width)
            .saturating_add(self.offset_col);
        let target_x = self.buffer.clamp_column(target_y, target_x);

        Some((target_y, target_x))
    }

    /// Posiciona el cursor a partir de coordenadas de pantalla (click simple de mouse).
    pub fn click_at(&mut self, screen_col: u16, screen_row: u16) {
        if let Some((y, x)) = self.resolve_click_position(screen_col, screen_row) {
            self.cursor_y = y;
            self.cursor_x = x;
        }
        self.clear_selection();
    }

    /// Inicia una selección en el punto donde se presionó el botón del mouse.
    pub fn start_selection_at(&mut self, screen_col: u16, screen_row: u16) {
        if let Some((y, x)) = self.resolve_click_position(screen_col, screen_row) {
            self.cursor_y = y;
            self.cursor_x = x;
            self.selection_anchor = Some((y, x));
        }
    }

    /// Extiende la selección activa hasta la posición arrastrada (drag de mouse).
    pub fn extend_selection_to(&mut self, screen_col: u16, screen_row: u16) {
        if self.selection_anchor.is_none() {
            return;
        }
        if let Some((y, x)) = self.resolve_click_position(screen_col, screen_row) {
            self.cursor_y = y;
            self.cursor_x = x;
        }
    }
    /// Desplaza la vista hacia arriba (rueda del mouse).
    pub fn scroll_up(&mut self, lines: usize) {
        self.offset_row = self.offset_row.saturating_sub(lines);
    }

    /// Desplaza la vista hacia abajo (rueda del mouse).
    pub fn scroll_down(&mut self, lines: usize) {
        let visible_lines = self.window_sizes.1.saturating_sub(4).max(1) as usize;
        let max_offset = self.buffer.line_count().saturating_sub(visible_lines);
        self.offset_row = (self.offset_row + lines).min(max_offset);
    }

    pub fn delete_forward_char(&mut self) {
        if self.delete_selection() {
            return;
        }
        self.push_undo_snapshot();
        self.buffer
            .delete_forward_char(self.cursor_y, self.cursor_x);
        self.search.clear();
    }

    pub fn adjust_scroll(&mut self) {
        let visible_lines = self.window_sizes.1.saturating_sub(4) as usize;

        if self.cursor_y < self.offset_row {
            self.offset_row = self.cursor_y;
        }

        if self.cursor_y >= self.offset_row + visible_lines {
            self.offset_row = self.cursor_y - visible_lines + 1;
        }

        let line_num_width = self.line_num_width();
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

        let visible_lines = height.saturating_sub(4) as usize;
        let line_count = self.buffer.line_count();
        let max_visible_lines = visible_lines.max(1);
        let max_offset_row = line_count.saturating_sub(max_visible_lines);

        if self.offset_row > self.cursor_y {
            self.offset_row = self.cursor_y;
        }
        if self.offset_row > max_offset_row {
            self.offset_row = max_offset_row;
        }

        let line_num_width = self.line_num_width();
        let visible_cols = width.saturating_sub(line_num_width as u16).max(1) as usize;
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
        self.replacement.clear();
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

        let clipboard = self.clipboard.clone();
        self.insert_text(&clipboard);
    }

    pub fn start_or_clear_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_y, self.cursor_x));
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn copy_selection(&mut self) {
        let Some(anchor) = self.selection_anchor else {
            self.state_msg = "Nada seleccionado".to_string();
            return;
        };

        let (start, end) = order_positions(anchor, (self.cursor_y, self.cursor_x));

        if start == end {
            self.state_msg = "Nada seleccionado".to_string();
            return;
        }

        self.clipboard = self.extract_range(start, end);
        self.state_msg = "Selección copiada".to_string();
    }

    fn extract_range(&self, start: (usize, usize), end: (usize, usize)) -> String {
        if start.0 == end.0 {
            let line = self.buffer.line(start.0);
            return line.chars().skip(start.1).take(end.1 - start.1).collect();
        }

        let mut result = String::new();
        for line_idx in start.0..=end.0 {
            let line = self.buffer.line(line_idx);
            let chars: Vec<char> = line.chars().collect();

            let slice: String = if line_idx == start.0 {
                chars[start.1..].iter().collect()
            } else if line_idx == end.0 {
                chars[..end.1.min(chars.len())].iter().collect()
            } else {
                chars.iter().collect()
            };

            result.push_str(&slice);
            if line_idx != end.0 {
                result.push('\n');
            }
        }

        result
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let cursor = (self.cursor_y, self.cursor_x);
        if anchor == cursor {
            return None;
        }

        Some(order_positions(anchor, cursor))
    }
    pub fn line_content(&self, idx: usize) -> String {
        self.buffer.line(idx)
    }
    pub fn write<W: Write>(&self, stdout: &mut W) {
        let mut out: Vec<u8> = Vec::with_capacity(16 * 1024);

        write!(out, "{}", cursor::Hide).unwrap();
        write!(out, "{}", cursor::MoveTo(0, 1)).unwrap();
        write!(
            out,
            "{}",
            terminal::Clear(terminal::ClearType::FromCursorDown)
        )
        .unwrap();

        let visible_lines = self.window_sizes.1.saturating_sub(4) as usize;
        let width = self.window_sizes.0 as usize;

        if visible_lines == 0 || self.window_sizes.0 == 0 {
            ui::render_message(&mut out, 1, width, "Ventana demasiado pequeña");
            write!(out, "{}", cursor::Show).unwrap();
            stdout.write_all(&out).unwrap();
            stdout.flush().unwrap();
            return;
        }
        let line_num_width = self.line_num_width();
        let language = ui::language_from_filename(self.filename.as_deref(), &self.language_config);

        let start = self.offset_row;
        let end = (self.offset_row + visible_lines).min(self.buffer.line_count());

        let selection = self.selection_range();

        for i in start..end {
            let line_num = i + 1;
            let window_row = 1 + (i - self.offset_row) as u16;

            if self.show_line_numbers {
                ui::render_line_number(&mut out, line_num, window_row, line_num_width);
            }
            let line = self.buffer.line(i);
            let visible_cols = width.saturating_sub(line_num_width);
            ui::render_line_content(
                &mut out,
                &line,
                i,
                ui::LineViewport {
                    start_col: self.offset_col,
                    visible_cols,
                },
                &self.search,
                ui::SyntaxRenderConfig {
                    language,
                    syntax_theme: &self.syntax_theme,
                },
                selection,
                self.ui_theme.selection_bg,
            );
        }

        let status_row = self.window_sizes.1.saturating_sub(3);
        let message_row = self.window_sizes.1.saturating_sub(2);
        let default_row = self.window_sizes.1.saturating_sub(1);
        ui::render_status_bar(
            &mut out,
            status_row,
            width,
            self.filename.as_deref(),
            self.cursor_y + 1,
            self.buffer.line_count(),
            self.cursor_x + 1,
            self.is_dirty(),
            self.ui_theme.status_bar_bg,
            self.ui_theme.status_bar_fg,
        );

        if self.state_msg != messages::DEFAULT_STATUS {
            ui::render_message(&mut out, message_row, width, &self.state_msg);
        } else {
            ui::render_message(&mut out, message_row, width, "");
        }
        ui::render_message(&mut out, default_row, width, messages::DEFAULT_STATUS);

        let (visual_x, visual_y) = self.cursor_screen_position();
        ui::position_cursor(&mut out, visual_x, visual_y);

        write!(out, "{}", cursor::Show).unwrap();

        stdout.write_all(&out).unwrap();
        stdout.flush().unwrap();
    }
}

fn order_positions(a: (usize, usize), b: (usize, usize)) -> ((usize, usize), (usize, usize)) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::Editor;

    #[test]
    fn insert_tab_adds_spaces_and_moves_cursor() {
        let mut editor = Editor::new();

        editor.insert_tab();

        assert_eq!(editor.buffer.line(0), "    ");
        assert_eq!(editor.cursor_x, 4);
        assert_eq!(editor.cursor_y, 0);
    }

    #[test]
    fn move_to_line_boundaries_updates_cursor() {
        let mut editor = Editor::new();
        editor.insert_char('h');
        editor.insert_char('o');
        editor.insert_char('l');
        editor.insert_char('a');

        editor.move_to_line_start();
        assert_eq!(editor.cursor_x, 0);

        editor.move_to_line_end();
        assert_eq!(editor.cursor_x, 4);
    }

    #[test]
    fn delete_forward_char_removes_character_under_cursor() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        editor.move_to_line_start();
        editor.move_right();

        editor.delete_forward_char();

        assert_eq!(editor.buffer.line(0), "ac");
        assert_eq!(editor.cursor_x, 1);
    }
}
