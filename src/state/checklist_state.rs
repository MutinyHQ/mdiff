#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub label: String,
    pub key: char, // Single-key shortcut (1-9, a-z)
    pub checked: bool,
    pub note: Option<String>, // Optional reviewer note
}

#[derive(Debug, Clone, Default)]
pub struct ChecklistState {
    pub items: Vec<ChecklistItem>,
    pub selected: usize,
    pub panel_open: bool,
}

impl ChecklistState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_config_items(config_items: Vec<(String, char)>) -> Self {
        let items = config_items
            .into_iter()
            .map(|(label, key)| ChecklistItem {
                label,
                key,
                checked: false,
                note: None,
            })
            .collect();

        Self {
            items,
            selected: 0,
            panel_open: false,
        }
    }

    pub fn select_up(&mut self) {
        if !self.items.is_empty() {
            if self.selected == 0 {
                self.selected = self.items.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    pub fn select_down(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn toggle_current_item(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected) {
            item.checked = !item.checked;
        }
    }

    pub fn set_current_note(&mut self, note: String) {
        if let Some(item) = self.items.get_mut(self.selected) {
            if note.trim().is_empty() {
                item.note = None;
            } else {
                item.note = Some(note.trim().to_string());
            }
        }
    }

    pub fn current_item(&self) -> Option<&ChecklistItem> {
        self.items.get(self.selected)
    }

    pub fn checked_count(&self) -> usize {
        self.items.iter().filter(|item| item.checked).count()
    }

    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    pub fn completion_percentage(&self) -> f32 {
        if self.items.is_empty() {
            100.0
        } else {
            (self.checked_count() as f32 / self.total_count() as f32) * 100.0
        }
    }

    pub fn reset(&mut self) {
        for item in &mut self.items {
            item.checked = false;
            item.note = None;
        }
        self.selected = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
