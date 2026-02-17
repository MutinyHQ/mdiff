pub mod action_hud;
pub mod commit_dialog;
pub mod context_bar;
pub mod diff_view;
pub mod navigator;
pub mod worktree_browser;

use ratatui::{layout::Rect, Frame};

use crate::state::AppState;

/// Trait for renderable TUI components.
pub trait Component {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);
}
