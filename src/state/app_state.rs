use super::{DiffOptions, DiffState, NavigatorState, WorktreeState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    DiffExplorer,
    WorktreeBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Navigator,
    DiffView,
}

pub struct AppState {
    pub active_view: ActiveView,
    pub focus: FocusPanel,
    pub diff: DiffState,
    pub navigator: NavigatorState,
    pub worktree: WorktreeState,
    pub should_quit: bool,
    pub commit_dialog_open: bool,
    pub commit_message: String,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub target_label: String,
}

impl AppState {
    pub fn new(diff_options: DiffOptions) -> Self {
        Self {
            active_view: ActiveView::DiffExplorer,
            focus: FocusPanel::Navigator,
            diff: DiffState::new(diff_options),
            navigator: NavigatorState::new(),
            worktree: WorktreeState::new(),
            should_quit: false,
            commit_dialog_open: false,
            commit_message: String::new(),
            status_message: None,
            target_label: String::new(),
        }
    }
}
