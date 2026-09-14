use crate::editor::Editor;

pub struct Tabs {
    editors: Vec<Editor>,
    active: usize,
}

impl Tabs {
    pub fn new() -> Self {
        Tabs {
            editors: vec![Editor::new()],
            active: 0,
        }
    }

    pub fn current(&self) -> &Editor {
        &self.editors[self.active]
    }

    pub fn current_mut(&mut self) -> &mut Editor {
        &mut self.editors[self.active]
    }

    pub fn new_tab(&mut self) {
        self.editors.push(Editor::new());
        self.active = self.editors.len() - 1;
    }

    /// Cierra la pestaña activa. Devuelve true si no quedan pestañas
    /// (el programa debería salir).
    pub fn close_current(&mut self) -> bool {
        self.editors.remove(self.active);
        if self.editors.is_empty() {
            return true;
        }
        if self.active >= self.editors.len() {
            self.active = self.editors.len() - 1;
        }
        false
    }

    pub fn next_tab(&mut self) {
        self.active = (self.active + 1) % self.editors.len();
    }

    pub fn previous_tab(&mut self) {
        self.active = (self.active + self.editors.len() - 1) % self.editors.len();
    }

    /// (etiqueta, es_activa, tiene_cambios_sin_guardar)
    pub fn labels(&self) -> Vec<(String, bool, bool)> {
        self.editors
            .iter()
            .enumerate()
            .map(|(idx, editor)| {
                let name = editor.filename.as_deref().unwrap_or("[Sin nombre]");
                (
                    format!("{}: {}", idx + 1, name),
                    idx == self.active,
                    editor.is_dirty(),
                )
            })
            .collect()
    }
}
