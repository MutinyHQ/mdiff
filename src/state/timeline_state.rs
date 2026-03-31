use crate::git::types::FileDelta;

#[derive(Debug, Clone)]
pub struct TimelineCommit {
    pub oid: String,
    pub summary: String,
    pub author: String,
    pub timestamp: String,
    pub files_changed: usize,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Default)]
pub struct TimelineState {
    pub active: bool,
    pub commits: Vec<TimelineCommit>,
    /// None = show all (combined diff), Some(i) = show only commit i's diff.
    pub selected_index: Option<usize>,
    /// Cached combined diff (the original full-range diff).
    pub cached_combined_deltas: Vec<FileDelta>,
}

impl TimelineState {
    pub fn select_next(&mut self) {
        if self.commits.is_empty() {
            return;
        }
        match self.selected_index {
            None => {
                self.selected_index = Some(0);
            }
            Some(i) => {
                if i + 1 < self.commits.len() {
                    self.selected_index = Some(i + 1);
                }
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.selected_index {
            None => {}
            Some(0) => {
                self.selected_index = None;
            }
            Some(i) => {
                self.selected_index = Some(i - 1);
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selected_index = None;
    }

    pub fn status_text(&self) -> String {
        if self.commits.is_empty() {
            return String::new();
        }
        match self.selected_index {
            None => format!("ALL ({} commits)", self.commits.len()),
            Some(i) => format!("[{}/{}]", i + 1, self.commits.len()),
        }
    }
}
