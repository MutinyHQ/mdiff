pub mod action_hud;
pub mod agent_outputs;
pub mod agent_selector;
pub mod annotation_menu;
pub mod comment_editor;
pub mod commit_dialog;
pub mod target_dialog;
pub mod context_bar;
pub mod diff_view;
pub mod navigator;
pub mod prompt_preview;
pub mod worktree_browser;

use ratatui::{layout::Rect, Frame};

use crate::state::AppState;

/// Trait for renderable TUI components.
pub trait Component {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);
}
