use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Bookmark {
    /// File path relative to repo root.
    pub file_path: String,
    /// Line number in the new file (preferred) or old file.
    pub line: u32,
    /// Whether this is a new-file or old-file line.
    pub is_new_line: bool,
    /// Optional single-character label (a-z, like vim marks).
    pub label: Option<char>,
    /// Timestamp for ordering.
    #[allow(dead_code)]
    pub created_at: u64,
}

impl Bookmark {
    pub fn new(file_path: String, line: u32, is_new_line: bool, label: Option<char>) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            file_path,
            line,
            is_new_line,
            label,
            created_at,
        }
    }

    /// Display-friendly location string.
    pub fn location(&self) -> String {
        format!("{}:{}", self.file_path, self.line)
    }
}

#[derive(Debug, Default)]
pub struct BookmarkState {
    /// All bookmarks in creation order.
    pub bookmarks: Vec<Bookmark>,
    /// Current bookmark index for navigation.
    pub current_index: Option<usize>,
    /// Whether the bookmark list panel is visible.
    pub list_visible: bool,
    /// Selected item in the bookmark list panel.
    pub list_selected: usize,
}

impl BookmarkState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a bookmark at the given file/line. Returns true if added, false if removed.
    pub fn toggle(&mut self, file_path: &str, line: u32, is_new_line: bool) -> bool {
        if let Some(idx) = self.find_at(file_path, line, is_new_line) {
            self.bookmarks.remove(idx);
            // Adjust current_index
            if let Some(ci) = self.current_index {
                if ci >= self.bookmarks.len() {
                    self.current_index = if self.bookmarks.is_empty() {
                        None
                    } else {
                        Some(self.bookmarks.len() - 1)
                    };
                } else if ci > idx {
                    self.current_index = Some(ci - 1);
                }
            }
            false
        } else {
            self.bookmarks.push(Bookmark::new(
                file_path.to_string(),
                line,
                is_new_line,
                None,
            ));
            true
        }
    }

    /// Set a named bookmark at the given location. If the label already exists, move it.
    pub fn set_named(&mut self, file_path: &str, line: u32, is_new_line: bool, label: char) {
        // Remove existing bookmark with this label
        self.bookmarks.retain(|b| b.label != Some(label));
        self.bookmarks.push(Bookmark::new(
            file_path.to_string(),
            line,
            is_new_line,
            Some(label),
        ));
    }

    /// Find the index of a bookmark at the given position.
    fn find_at(&self, file_path: &str, line: u32, is_new_line: bool) -> Option<usize> {
        self.bookmarks.iter().position(|b| {
            b.file_path == file_path && b.line == line && b.is_new_line == is_new_line
        })
    }

    /// Find a named bookmark by its label character.
    pub fn find_named(&self, label: char) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.label == Some(label))
    }

    /// Check if there is a bookmark at the given line position.
    #[allow(dead_code)]
    pub fn has_bookmark_at(
        &self,
        file_path: &str,
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
    ) -> bool {
        self.bookmarks.iter().any(|b| {
            b.file_path == file_path
                && ((b.is_new_line && new_lineno == Some(b.line))
                    || (!b.is_new_line && old_lineno == Some(b.line)))
        })
    }

    /// Get the bookmark label at a given line position (for gutter display).
    pub fn label_at(
        &self,
        file_path: &str,
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
    ) -> Option<Option<char>> {
        self.bookmarks.iter().find_map(|b| {
            if b.file_path == file_path
                && ((b.is_new_line && new_lineno == Some(b.line))
                    || (!b.is_new_line && old_lineno == Some(b.line)))
            {
                Some(b.label)
            } else {
                None
            }
        })
    }

    /// Find the next bookmark after the given file/line position.
    pub fn next_after(&self, file_path: &str, line: u32) -> Option<(usize, &Bookmark)> {
        let mut sorted: Vec<(usize, &Bookmark)> = self.bookmarks.iter().enumerate().collect();
        sorted.sort_by(|(_, a), (_, b)| a.file_path.cmp(&b.file_path).then(a.line.cmp(&b.line)));

        // Find first bookmark after current position
        for (idx, bm) in &sorted {
            if bm.file_path.as_str() > file_path || (bm.file_path == file_path && bm.line > line) {
                return Some((*idx, bm));
            }
        }
        // Wrap around to first
        sorted.first().map(|(idx, bm)| (*idx, *bm))
    }

    /// Find the previous bookmark before the given file/line position.
    pub fn prev_before(&self, file_path: &str, line: u32) -> Option<(usize, &Bookmark)> {
        let mut sorted: Vec<(usize, &Bookmark)> = self.bookmarks.iter().enumerate().collect();
        sorted.sort_by(|(_, a), (_, b)| a.file_path.cmp(&b.file_path).then(a.line.cmp(&b.line)));

        // Find last bookmark before current position
        for (idx, bm) in sorted.iter().rev() {
            if bm.file_path.as_str() < file_path || (bm.file_path == file_path && bm.line < line) {
                return Some((*idx, *bm));
            }
        }
        // Wrap around to last
        sorted.last().map(|(idx, bm)| (*idx, *bm))
    }

    /// Delete the bookmark at the given index.
    pub fn delete(&mut self, idx: usize) -> bool {
        if idx < self.bookmarks.len() {
            self.bookmarks.remove(idx);
            if self.list_selected >= self.bookmarks.len() && !self.bookmarks.is_empty() {
                self.list_selected = self.bookmarks.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Total bookmark count.
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Count bookmarks in a specific file.
    #[allow(dead_code)]
    pub fn count_in_file(&self, file_path: &str) -> usize {
        self.bookmarks
            .iter()
            .filter(|b| b.file_path == file_path)
            .count()
    }

    /// Get all bookmarks sorted by file path then line number.
    pub fn all_sorted(&self) -> Vec<(usize, &Bookmark)> {
        let mut sorted: Vec<(usize, &Bookmark)> = self.bookmarks.iter().enumerate().collect();
        sorted.sort_by(|(_, a), (_, b)| a.file_path.cmp(&b.file_path).then(a.line.cmp(&b.line)));
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_adds_and_removes() {
        let mut state = BookmarkState::new();
        assert!(state.toggle("foo.rs", 10, true));
        assert_eq!(state.count(), 1);
        assert!(!state.toggle("foo.rs", 10, true));
        assert_eq!(state.count(), 0);
    }

    #[test]
    fn named_bookmark_moves_on_reassign() {
        let mut state = BookmarkState::new();
        state.set_named("foo.rs", 10, true, 'a');
        assert_eq!(state.count(), 1);
        state.set_named("bar.rs", 20, true, 'a');
        assert_eq!(state.count(), 1);
        let bm = state.find_named('a').unwrap();
        assert_eq!(bm.file_path, "bar.rs");
        assert_eq!(bm.line, 20);
    }

    #[test]
    fn has_bookmark_at_checks_correctly() {
        let mut state = BookmarkState::new();
        state.toggle("foo.rs", 10, true);
        assert!(state.has_bookmark_at("foo.rs", None, Some(10)));
        assert!(!state.has_bookmark_at("foo.rs", None, Some(11)));
        assert!(!state.has_bookmark_at("bar.rs", None, Some(10)));
    }

    #[test]
    fn next_and_prev_navigation() {
        let mut state = BookmarkState::new();
        state.toggle("a.rs", 5, true);
        state.toggle("a.rs", 15, true);
        state.toggle("b.rs", 3, true);

        let (_, bm) = state.next_after("a.rs", 5).unwrap();
        assert_eq!(bm.line, 15);

        let (_, bm) = state.next_after("a.rs", 15).unwrap();
        assert_eq!(bm.file_path, "b.rs");

        let (_, bm) = state.prev_before("b.rs", 3).unwrap();
        assert_eq!(bm.line, 15);
    }
}
