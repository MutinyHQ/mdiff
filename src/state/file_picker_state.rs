use super::TextBuffer;

/// An entry in the file picker, storing path and change stats.
#[derive(Debug, Clone)]
pub struct FilePickerEntry {
    pub delta_index: usize,
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
}

/// A filtered result with match indices for highlighting.
#[derive(Debug, Clone)]
pub struct FilteredEntry {
    pub entry_index: usize,
    pub score: u32,
    pub match_indices: Vec<u32>,
}

pub struct FilePickerState {
    pub active: bool,
    pub query: TextBuffer,
    pub entries: Vec<FilePickerEntry>,
    pub filtered: Vec<FilteredEntry>,
    pub selected: usize,
}

impl FilePickerState {
    pub fn new() -> Self {
        Self {
            active: false,
            query: TextBuffer::new(),
            entries: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
        }
    }

    pub fn open(&mut self, entries: Vec<FilePickerEntry>) {
        self.active = true;
        self.query.clear();
        self.entries = entries;
        self.filtered = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, _)| FilteredEntry {
                entry_index: i,
                score: 0,
                match_indices: Vec::new(),
            })
            .collect();
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.query.clear();
        self.entries.clear();
        self.filtered.clear();
        self.selected = 0;
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected < self.filtered.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn selected_delta_index(&self) -> Option<usize> {
        self.filtered
            .get(self.selected)
            .and_then(|f| self.entries.get(f.entry_index))
            .map(|e| e.delta_index)
    }
}

impl Default for FilePickerState {
    fn default() -> Self {
        Self::new()
    }
}
