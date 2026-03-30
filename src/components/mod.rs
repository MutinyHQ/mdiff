pub mod action_hud;
pub mod agent_outputs;
pub mod agent_selector;
pub mod agentic_review_panel;
pub mod annotation_menu;
pub mod bookmark_list;
pub mod category_picker;
pub mod checklist_panel;
pub mod command_bar;
pub mod comment_editor;
pub mod commit_dialog;
pub mod context_bar;
pub mod diff_view;
pub mod feedback_summary;
pub mod file_picker;
pub mod global_search_bar;
pub mod navigator;
pub mod prompt_preview;
pub mod qa_answer;
pub mod qa_input;
pub mod restore_confirm;
pub mod settings_modal;
pub mod target_dialog;
pub mod text_input;
pub mod which_key;
pub mod worktree_browser;

use ratatui::{layout::Rect, Frame};

use crate::state::AppState;

/// Trait for renderable TUI components.
pub trait Component {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);
}
