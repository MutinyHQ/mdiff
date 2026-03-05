use crate::git::types::FileDelta;
use std::collections::{HashMap, HashSet};

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
        let paths: Vec<String> = deltas
            .iter()
            .map(|d| d.path.to_string_lossy().to_string())
            .collect();
        let informative_paths = build_informative_path_displays(&paths);

        self.entries = deltas
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let path_str = paths[i].clone();
                let display = format!(
                    "{} [{}] +{} -{}",
                    informative_paths[i],
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

fn build_informative_path_displays(paths: &[String]) -> Vec<String> {
    let split_paths: Vec<Vec<String>> = paths
        .iter()
        .map(|p| p.split('/').map(str::to_string).collect())
        .collect();

    let mut sibling_map: HashMap<String, HashSet<String>> = HashMap::new();
    for components in &split_paths {
        if components.len() < 2 {
            continue;
        }
        for i in 0..(components.len() - 1) {
            let parent = components[..i].join("/");
            sibling_map
                .entry(parent)
                .or_default()
                .insert(components[i].clone());
        }
    }

    split_paths
        .into_iter()
        .map(|components| abbreviate_path_components(&components, &sibling_map))
        .collect()
}

fn abbreviate_path_components(
    components: &[String],
    sibling_map: &HashMap<String, HashSet<String>>,
) -> String {
    if components.len() <= 1 {
        return components.first().cloned().unwrap_or_default();
    }

    let mut out = Vec::with_capacity(components.len());
    for i in 0..(components.len() - 1) {
        let parent = components[..i].join("/");
        let siblings = sibling_map.get(&parent);
        let abbreviated = minimal_unique_prefix(&components[i], siblings);
        out.push(abbreviated);
    }

    // Keep filename fully readable; only directory components are abbreviated.
    out.push(components.last().cloned().unwrap_or_default());
    out.join("/")
}

fn minimal_unique_prefix(name: &str, siblings: Option<&HashSet<String>>) -> String {
    let Some(siblings) = siblings else {
        return first_char_or_empty(name);
    };
    if siblings.len() <= 1 {
        return first_char_or_empty(name);
    }

    let char_count = name.chars().count();
    for len in 1..=char_count {
        let prefix = take_chars(name, len);
        let unique = siblings
            .iter()
            .filter(|s| s.as_str() != name)
            .all(|s| !s.starts_with(&prefix));
        if unique {
            return prefix;
        }
    }

    name.to_string()
}

fn first_char_or_empty(s: &str) -> String {
    s.chars().next().map(|c| c.to_string()).unwrap_or_default()
}

fn take_chars(s: &str, count: usize) -> String {
    s.chars().take(count).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::{FileDelta, FileStatus};
    use std::path::PathBuf;

    fn make_delta(path: &str, status: FileStatus, additions: usize, deletions: usize) -> FileDelta {
        FileDelta {
            path: PathBuf::from(path),
            old_path: None,
            status,
            hunks: Vec::new(),
            additions,
            deletions,
            binary: false,
        }
    }

    #[test]
    fn abbreviates_by_minimal_unique_prefix_under_same_parent() {
        let paths = vec![
            "src/components/navigator.rs".to_string(),
            "src/config/navigator.rs".to_string(),
        ];
        let displays = build_informative_path_displays(&paths);
        assert_eq!(displays[0], "s/com/navigator.rs");
        assert_eq!(displays[1], "s/con/navigator.rs");
    }

    #[test]
    fn uses_single_char_for_non_conflicting_directories() {
        let paths = vec!["docs/readme.md".to_string()];
        let displays = build_informative_path_displays(&paths);
        assert_eq!(displays[0], "d/readme.md");
    }

    #[test]
    fn preserves_filename_component() {
        let paths = vec![
            "src/components/navigator_super_long_name.rs".to_string(),
            "src/config/navigator_super_long_name.rs".to_string(),
        ];
        let displays = build_informative_path_displays(&paths);
        assert!(displays[0].ends_with("/navigator_super_long_name.rs"));
        assert!(displays[1].ends_with("/navigator_super_long_name.rs"));
    }

    #[test]
    fn update_from_deltas_uses_informative_path_in_display() {
        let deltas = vec![
            make_delta("src/components/navigator.rs", FileStatus::Modified, 12, 4),
            make_delta("src/config/navigator.rs", FileStatus::Added, 8, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);

        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].display, "s/com/navigator.rs [M] +12 -4");
        assert_eq!(state.entries[1].display, "s/con/navigator.rs [A] +8 -0");
        assert_eq!(state.entries[0].path, "src/components/navigator.rs");
        assert_eq!(state.entries[1].path, "src/config/navigator.rs");
    }
}
