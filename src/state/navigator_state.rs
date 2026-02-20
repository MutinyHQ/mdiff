use crate::git::types::FileDelta;

use super::TextBuffer;

#[derive(Debug)]
pub struct NavigatorEntry {
    pub display: String,
    pub path: String,
    pub delta_index: usize,
}

#[derive(Debug)]
pub struct NavigatorState {
    pub selected: usize,
    pub entries: Vec<NavigatorEntry>,
    pub filtered_indices: Vec<usize>,
    pub search_active: bool,
    pub search_query: TextBuffer,
    /// Saved selection index before search started (for cancel/restore).
    pre_search_selected: Option<usize>,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            entries: Vec::new(),
            filtered_indices: Vec::new(),
            search_active: false,
            search_query: TextBuffer::new(),
            pre_search_selected: None,
        }
    }

    pub fn update_from_deltas(&mut self, deltas: &[FileDelta]) {
        self.entries = deltas
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let path_str = d.path.to_string_lossy().to_string();
                let display = format!(
                    "{} [{}] +{} -{}",
                    path_str,
                    d.status.label(),
                    d.additions,
                    d.deletions
                );
                NavigatorEntry {
                    display,
                    path: path_str,
                    delta_index: i,
                }
            })
            .collect();

        self.refilter();
    }

    pub fn refilter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
        } else {
            let query_lower = self.search_query.text().to_lowercase();
            self.filtered_indices = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| fuzzy_match(&e.path.to_lowercase(), &query_lower))
                .map(|(i, _)| i)
                .collect();
        }

        // Clamp selection
        if !self.filtered_indices.is_empty() {
            self.selected = self.selected.min(self.filtered_indices.len() - 1);
        } else {
            self.selected = 0;
        }
    }

    pub fn visible_entries(&self) -> Vec<(usize, &NavigatorEntry)> {
        self.filtered_indices
            .iter()
            .map(|&i| (i, &self.entries[i]))
            .collect()
    }

    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_indices.len() - 1);
        }
    }

    pub fn selected_delta_index(&self) -> Option<usize> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&i| self.entries.get(i))
            .map(|e| e.delta_index)
    }

    pub fn start_search(&mut self) {
        self.pre_search_selected = Some(self.selected);
        self.search_active = true;
        self.search_query.clear();
    }

    /// Confirm search (Enter): resolve the currently selected entry, then
    /// clear the query and refilter so all entries are visible again, keeping
    /// focus on the entry that was selected in the filtered list.
    pub fn confirm_search(&mut self) {
        let target_delta_index = self.selected_delta_index();
        self.search_active = false;
        self.search_query.clear();
        self.refilter();
        // Find the entry with the same delta_index in the now-unfiltered list
        if let Some(delta_idx) = target_delta_index {
            if let Some(pos) = self
                .filtered_indices
                .iter()
                .position(|&i| self.entries[i].delta_index == delta_idx)
            {
                self.selected = pos;
            }
        }
        self.pre_search_selected = None;
    }

    /// Cancel search (Esc): restore the selection from before search started.
    pub fn cancel_search(&mut self) {
        let restore = self.pre_search_selected.take();
        self.search_active = false;
        self.search_query.clear();
        self.refilter();
        if let Some(prev) = restore {
            self.selected = prev.min(self.filtered_indices.len().saturating_sub(1));
        }
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.insert_char(c);
        self.selected = 0;
        self.refilter();
    }

    pub fn search_pop(&mut self) {
        self.search_query.delete_back();
        self.selected = 0;
        self.refilter();
    }
}

/// Simple fuzzy match: all characters of pattern must appear in text in order.
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let mut text_iter = text.chars();
    for pc in pattern.chars() {
        loop {
            match text_iter.next() {
                Some(tc) if tc == pc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}
