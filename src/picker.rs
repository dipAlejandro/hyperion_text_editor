use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickerEntryKind {
    Dir,
    File,
}

pub struct PickerEntry {
    pub name: String,
    pub kind: PickerEntryKind,
    pub match_indices: Vec<usize>,
}

pub struct FilePicker {
    current_dir: PathBuf,
    entries: Vec<(String, PickerEntryKind)>,
    matcher: SkimMatcherV2,
    query: String,
    filtered: Vec<PickerEntry>,
    selected: usize,
    scroll_offset: usize,
}

impl FilePicker {
    pub fn open(start_dir: &Path) -> Self {
        let mut picker = FilePicker {
            current_dir: start_dir.to_path_buf(),
            entries: Vec::new(),
            matcher: SkimMatcherV2::default(),
            query: String::new(),
            filtered: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        };
        picker.read_dir();
        picker
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn entries(&self) -> &[PickerEntry] {
        &self.filtered
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.filter();
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.filter();
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected - visible_rows + 1;
        }
    }

    /// Activa la entrada seleccionada. Si es un directorio, navega y
    /// devuelve None. Si es un archivo, devuelve su ruta.
    pub fn activate(&mut self) -> Option<PathBuf> {
        let entry = self.filtered.get(self.selected)?;

        match entry.kind {
            PickerEntryKind::Dir => {
                let target = if entry.name == ".." {
                    self.current_dir.parent()?.to_path_buf()
                } else {
                    self.current_dir.join(&entry.name)
                };
                self.current_dir = target;
                self.read_dir();
                None
            }
            PickerEntryKind::File => Some(self.current_dir.join(&entry.name)),
        }
    }

    fn read_dir(&mut self) {
        self.entries.clear();

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        if let Ok(read) = std::fs::read_dir(&self.current_dir) {
            for entry in read.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                match entry.file_type() {
                    Ok(ft) if ft.is_dir() => dirs.push(name),
                    Ok(ft) if ft.is_file() => files.push(name),
                    _ => {}
                }
            }
        }

        dirs.sort();
        files.sort();

        if self.current_dir.parent().is_some() {
            self.entries.push(("..".to_string(), PickerEntryKind::Dir));
        }
        for d in dirs {
            self.entries.push((d, PickerEntryKind::Dir));
        }
        for f in files {
            self.entries.push((f, PickerEntryKind::File));
        }

        self.query.clear();
        self.filter();
    }

    fn filter(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;

        if self.query.is_empty() {
            self.filtered = self
                .entries
                .iter()
                .map(|(name, kind)| PickerEntry {
                    name: name.clone(),
                    kind: *kind,
                    match_indices: Vec::new(),
                })
                .collect();
            return;
        }

        let mut scored: Vec<(i64, PickerEntry)> = self
            .entries
            .iter()
            .filter_map(|(name, kind)| {
                let (score, indices) = self.matcher.fuzzy_indices(name, &self.query)?;
                Some((
                    score,
                    PickerEntry {
                        name: name.clone(),
                        kind: *kind,
                        match_indices: indices,
                    },
                ))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered = scored.into_iter().map(|(_, e)| e).collect();
    }
}
