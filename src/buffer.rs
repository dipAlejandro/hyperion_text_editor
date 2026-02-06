use ropey::Rope;
use std::fs;

/// Representa el buffer de texto del documento
///
/// Un buffer contiene las líneas del documento y proporciona
/// operaciones para manipular el texto
pub struct TextBuffer {
    rope: Rope,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer { rope: Rope::new() }
    }

    /// Crea un buffer desde un archivo
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let mut content = fs::read_to_string(path)?;

        if content.is_empty() || !content.ends_with('\n') {
            content.push('\n');
        }

        Ok(Self {
            rope: Rope::from_str(&content),
        })
    }

    /// Guarda el buffer en un archivo
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        fs::write(path, self.rope.to_string())
    }

    /// Obtiene la linea perteneciente al indice indicado (sin \n final)
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea (base 0)
    ///
    /// # Retorna
    /// La linea como String, sin el carácter de salto de línea
    pub fn line(&self, idx: usize) -> String {
        let line = self.rope.line(idx);
        let s = line.to_string();

        // Quitar el \n final si existe
        if s.ends_with('\n') {
            s[..s.len() - 1].to_string()
        } else {
            s
        }
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = String> + '_ {
        (0..self.line_count()).map(|i| self.line(i))
    }

    /// Obtiene el numero total de lineas
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Obtiene la longitud de una línea específica (sin contar el \n final)
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea (base 0)
    ///
    /// # Retorna
    /// Longitud de la línea en caracteres, excluyendo el salto de línea
    pub fn line_length(&self, line_idx: usize) -> usize {
        let line = self.rope.line(line_idx);
        let len = line.len_chars();

        // Si la línea termina con \n, no contarlo en la longitud
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }
    /// Inserta un carácter en una posición específica
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea donde insertar
    /// * `col` - Columna donde insertar el carácter
    /// * `ch` - Carácter a insertar
    pub fn insert_char(&mut self, line_idx: usize, col: usize, ch: char) {
        // Obtener la posición de inicio de la línea en chars
        let line_start = self.rope.line_to_char(line_idx);

        // Obtener la longitud real de la línea (sin \n)
        let line_len = self.line_length(line_idx);

        // Asegurar que col esté dentro de los límites
        let safe_col = col.min(line_len);

        // Calcular la posición absoluta en el rope
        let char_idx = line_start + safe_col;

        // Insertar el carácter
        self.rope.insert_char(char_idx, ch);
    }

    /// Inserta una cadena en una posición específica
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea donde insertar
    /// * `col` - Columna donde insertar la cadena
    /// * `text` - Texto a insertar
    pub fn insert_str(&mut self, line_idx: usize, col: usize, text: &str) {
        let line_start = self.rope.line_to_char(line_idx);
        let line_len = self.line_length(line_idx);
        let safe_col = col.min(line_len);
        let char_idx = line_start + safe_col;

        self.rope.insert(char_idx, text);
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

        let line_len = self.line_length(line_idx);

        if col > line_len {
            return false;
        }

        let line_start = self.rope.line_to_char(line_idx);
        let char_idx = line_start + col - 1;

        self.rope.remove(char_idx..char_idx + 1);

        true
    }
    /// Une la línea actual con la anterior
    ///
    /// # Argumentos
    /// * `line_idx` - Índice de la línea a unir con la anterior
    ///
    /// # Retorna
    /// La longitud de la línea anterior antes de unir (nueva posición del cursor)
    pub fn join_with_previous(&mut self, line_idx: usize) -> usize {
        if line_idx == 0 {
            return 0;
        }

        let prev_len = self.line_length(line_idx - 1);

        // Encontrar el \n al final de la línea anterior
        let prev_line_start = self.rope.line_to_char(line_idx - 1);
        let newline_pos = prev_line_start + prev_len;

        // Eliminar el \n
        self.rope.remove(newline_pos..newline_pos + 1);

        prev_len
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
        let line_start = self.rope.line_to_char(line_idx);
        let line_len = self.line_length(line_idx);
        let safe_col = col.min(line_len);

        let char_idx = line_start + safe_col;
        self.rope.insert_char(char_idx, '\n');

        (line_idx + 1, 0)
    }

    /// Verifica si un índice de línea es válido
    pub fn is_valid_line(&self, line_idx: usize) -> bool {
        line_idx < self.line_count()
    }

    /// Ajusta una columna para que esté dentro de los límites de una línea
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

        assert_eq!(buffer.line(0), "hi");
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char(0, 0, 'h');
        buffer.insert_char(0, 1, 'i');

        let deleted = buffer.delete_char(0, 2);
        assert!(deleted);
        assert_eq!(buffer.line(0), "h");
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
        assert_eq!(buffer.line(0), "he");
        assert_eq!(buffer.line(1), "llo");
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
        assert_eq!(buffer.line(0), "hiby");
        assert_eq!(prev_len, 2);
    }
}
