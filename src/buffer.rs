use std::fs;

/// Representa el buffer de texto del documento
///
/// Un buffer contiene las líneas del documento y proporciona
/// operaciones para manipular el texto
pub struct TextBuffer {
    lines: Vec<String>,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            lines: vec![String::new()],
        }
    }

    /// Crea un buffer desde un archivo
    ///
    /// # Argumentos
    /// * `path` - Ruta del archivo a cargar
    ///
    /// # Retorna
    /// Result con el buffer cargado o un error de IO
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();

        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Ok(TextBuffer { lines })
    }

    /// Guarda el buffer en un archivo
    ///
    /// # Argumentos
    /// * `path` - Ruta donde guardar el archivo
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let content = self.lines.join("\n");
        fs::write(path, content)
    }

    /// Obtiene una referencia a todas las lineas
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Obtiene el numero total de lineas
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Obtiene la longitud de una línea específica
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea (base 0)
    ///
    /// # Retorna
    /// Longitud de la línea, o 0 si el índice es inválido
    pub fn line_length(&self, line_idx: usize) -> usize {
        self.lines.get(line_idx).map(|l| l.len()).unwrap_or(0)
    }

    /// Inserta un carácter en una posición específica
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea donde insertar
    /// * `col` - Columna donde insertar el carácter
    /// * `ch` - Carácter a insertar
    ///
    /// # Panics
    /// Si el índice de línea es inválido
    pub fn insert_char(&mut self, line_idx: usize, col: usize, ch: char) {
        let line = &mut self.lines[line_idx];
        let byte_idx = char_col_to_byte_idx(line, col);
        line.insert(byte_idx, ch);
    }

    /// Elimina el carácter antes de la posición especificada
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea
    /// * `col` - Columna donde está el cursor
    ///
    /// # Retorna
    /// `true` si se eliminó un carácter, `false` si no había nada que eliminar
    pub fn delete_char(&mut self, line_idx: usize, col: usize) -> bool {
        if col == 0 {
            return false;
        }

        let line = &mut self.lines[line_idx];
        let byte_start = char_col_to_byte_idx(line, col - 1);
        let byte_end = char_col_to_byte_idx(line, col);

        line.replace_range(byte_start..byte_end, "");
        true
    }

    /// Une la línea actual con la anterior
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea a unir con la anterior
    ///
    /// # Retorna
    /// La longitud de la línea anterior antes de unir (nueva posición del cursor)
    ///
    /// # Panics
    /// Si line_idx es 0 o inválido
    pub fn join_with_previous(&mut self, line_idx: usize) -> usize {
        let current_line = self.lines.remove(line_idx);
        let previous_len = self.lines[line_idx - 1].len();
        self.lines[line_idx - 1].push_str(&current_line);
        previous_len
    }

    /// Divide una línea en dos en la posición del cursor
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea a dividir
    /// * `col` - Columna donde dividir
    ///
    /// # Retorna
    /// Una tupla con (nuevo_line_idx, nueva_col) para el cursor
    pub fn split_line(&mut self, line_idx: usize, col: usize) -> (usize, usize) {
        let line = &mut self.lines[line_idx];
        let byte_idx = char_col_to_byte_idx(line, col);

        let right_txt = line[byte_idx..].to_string();
        self.lines[line_idx].truncate(byte_idx);
        self.lines.insert(line_idx + 1, right_txt);

        (line_idx + 1, 0)
    }

    /// Verifica si un índice de línea es válido
    pub fn is_valid_line(&self, line_idx: usize) -> bool {
        line_idx < self.lines.len()
    }

    /// Ajusta una columna para que esté dentro de los límites de una línea
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea
    /// * `col` - Columna a ajustar
    ///
    /// # Retorna
    /// La columna ajustada (no mayor que la longitud de la línea)
    pub fn clamp_column(&self, line_idx: usize, col: usize) -> usize {
        let line_len = self.line_length(line_idx);
        col.min(line_len)
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

fn char_col_to_byte_idx(s: &str, col: usize) -> usize {
    s.char_indices()
        .nth(col)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_has_one_line() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line_length(0), 0);
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char(0, 0, 'h');
        buffer.insert_char(0, 1, 'i');

        assert_eq!(buffer.lines()[0], "hi");
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char(0, 0, 'h');
        buffer.insert_char(0, 1, 'i');

        let deleted = buffer.delete_char(0, 2);
        assert!(deleted);
        assert_eq!(buffer.lines()[0], "h");
    }

    #[test]
    fn test_split_line() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char(0, 0, 'h');
        buffer.insert_char(0, 1, 'e');
        buffer.insert_char(0, 2, 'l');
        buffer.insert_char(0, 3, 'l');
        buffer.insert_char(0, 4, 'o');

        let (new_line, new_col) = buffer.split_line(0, 2);

        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.lines()[0], "he");
        assert_eq!(buffer.lines()[1], "llo");
        assert_eq!(new_line, 1);
        assert_eq!(new_col, 0);
    }

    #[test]
    fn test_join_with_previous() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char(0, 0, 'h');
        buffer.insert_char(0, 1, 'i');
        buffer.split_line(0, 2);
        buffer.insert_char(1, 0, 'b');
        buffer.insert_char(1, 1, 'y');

        let prev_len = buffer.join_with_previous(1);

        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.lines()[0], "hiby");
        assert_eq!(prev_len, 2);
    }
}
