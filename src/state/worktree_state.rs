use crate::git::worktree::WorktreeInfo;

#[derive(Debug)]
pub struct WorktreeState {
    pub selected: usize,
    pub worktrees: Vec<WorktreeInfo>,
    pub loading: bool,
}

impl WorktreeState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            worktrees: Vec::new(),
            loading: false,
        }
    }

    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_down(&mut self) {
        if !self.worktrees.is_empty() {
            self.selected = (self.selected + 1).min(self.worktrees.len() - 1);
        }
    }

    pub fn selected_worktree(&self) -> Option<&WorktreeInfo> {
        self.worktrees.get(self.selected)
    }
}
