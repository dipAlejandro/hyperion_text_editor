/// Representa coincidencia de busqueda en el documento
#[derive(Clone, Debug)]
pub struct Match {
    /// Indice de la linea donde aparece la coincidencia
    pub line: usize,

    /// Columna donde inicia la coincidencia
    pub start_col: usize,

    /// Columna donde termina la coincidencia
    pub end_col: usize,
}

impl Match {
    pub fn new(line: usize, start_col: usize, end_col: usize) -> Self {
        Match {
            line,
            start_col,
            end_col,
        }
    }
}

/// Gestor de estado de busqueda
pub struct SearchState {
    query: Option<String>,
    matches: Vec<Match>,
    current_index: Option<usize>,
}

impl SearchState {
    pub fn new() -> Self {
        SearchState {
            query: None,
            matches: Vec::new(),
            current_index: None,
        }
    }

    /// Obtener la consulta actual
    pub fn query(&self) -> Option<&String> {
        self.query.as_ref()
    }

    /// Obtener todas las coincidencia
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    /// Obtener el indice de la coincidencia actual
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Obtener la coincidencia actual
    pub fn current_match(&self) -> Option<&Match> {
        self.current_index.and_then(|idx| self.matches.get(idx))
    }

    /// Verificar si hay busqueda activa
    pub fn is_active(&self) -> bool {
        self.query.is_some()
    }

    /// Cuenta el numero de coincidencias
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Realiza una búsqueda en las líneas proporcionadas
    ///
    /// # Argumentos
    /// * `query` - Texto a buscar
    /// * `lines` - Líneas del documento donde buscar
    ///
    /// # Retorna
    /// Número de coincidencias encontradas
    pub fn search(&mut self, query: &str, lines: &[String]) -> usize {
        // Limpiar estado anterior
        self.matches.clear();
        self.current_index = None;

        // Si la búsqueda está vacía, salir
        if query.is_empty() {
            self.query = None;
            return 0;
        }

        // Guardar la consulta
        self.query = Some(query.to_string());
        let query_len = query.chars().count();

        // Buscar en todas las líneas
        for (line_idx, line) in lines.iter().enumerate() {
            let mut char_col = 0;

            // Buscar todas las ocurrencias en esta línea usando índices por carácter
            for (byte_idx, _) in line.char_indices() {
                if line[byte_idx..].starts_with(query) {
                    self.matches.push(Match::new(
                        line_idx,
                        char_col,
                        char_col + query_len,
                    ));
                }
                char_col += 1;
            }
        }

        // Si hay coincidencias, posicionarse en la primera
        if !self.matches.is_empty() {
            self.current_index = Some(0);
        }

        self.matches.len()
    }

    /// Avanza a la siguiente coincidencia (circular)
    ///
    /// # Retorna
    /// La siguiente coincidencia, o None si no hay busqueda activa
    pub fn next_match(&mut self) -> Option<&Match> {
        if self.matches.is_empty() {
            return None;
        }

        if let Some(current_idx) = self.current_index {
            let next_idx = (current_idx + 1) % self.matches.len();
            self.current_index = Some(next_idx);

            self.matches.get(next_idx)
        } else {
            None
        }
    }

    /// Retrocede a la coincidencia anterior (circular)
    ///
    /// # Retorna
    /// La coincidencia anterior, o None si no hay búsqueda activa
    pub fn previous_match(&mut self) -> Option<&Match> {
        if self.matches.is_empty() {
            return None;
        }

        if let Some(current_idx) = self.current_index {
            let total = self.matches.len();
            let prev_idx = (current_idx + total - 1) % total;
            self.current_index = Some(prev_idx);
            self.matches.get(prev_idx)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn test_search_find_matches() {
        let lines = vec![
            "hello world".to_string(),
            "hello rust".to_string(),
            "goodbye world".to_string(),
        ];

        let mut state = SearchState::new();
        let count = state.search("hello", &lines);

        assert_eq!(count, 2);
        assert_eq!(state.match_count(), 2);
    }

    #[test]
    fn test_circular_navegation() {
        let lines = vec![" a b a".to_string()];
        let mut state = SearchState::new();

        state.search("a", &lines);

        assert_eq!(state.current_index, Some(0));

        state.next_match();
        assert_eq!(state.current_index, Some(1));

        state.previous_match();
        assert_eq!(state.current_index, Some(0));
    }

    #[test]
    fn test_search_utf8_overlapping() {
        let lines = vec!["ááá".to_string()];
        let mut state = SearchState::new();

        let count = state.search("áá", &lines);

        assert_eq!(count, 2);
        assert_eq!(state.matches[0].start_col, 0);
        assert_eq!(state.matches[0].end_col, 2);
        assert_eq!(state.matches[1].start_col, 1);
        assert_eq!(state.matches[1].end_col, 3);
    }
}
