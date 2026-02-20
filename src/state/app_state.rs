use crate::theme::Theme;

use super::{
    AgentOutputsState, AgentSelectorState, AnnotationState, DiffOptions, DiffState, NavigatorState,
    ReviewState, SelectionState, TextBuffer, WorktreeState,
};

use super::settings_state::SettingsState;

/// Snapshot of an annotation for the annotation menu (owned to avoid borrow issues).
#[derive(Debug, Clone)]
pub struct AnnotationMenuItem {
    pub file_path: String,
    pub old_range: Option<(u32, u32)>,
    pub new_range: Option<(u32, u32)>,
    pub comment: String,
}

impl AnnotationMenuItem {
    /// Representative line for display, preferring new-file.
    pub fn sort_line(&self) -> u32 {
        self.new_range
            .map(|(s, _)| s)
            .or(self.old_range.map(|(s, _)| s))
            .unwrap_or(0)
    }

    /// Format a human-readable range string.
    pub fn range_text(&self) -> String {
        match (self.old_range, self.new_range) {
            (_, Some((s, e))) if s == e => format!("Line {s}"),
            (_, Some((s, e))) => format!("Lines {s}-{e}"),
            (Some((s, e)), None) if s == e => format!("Removed line {s} (old)"),
            (Some((s, e)), None) => format!("Removed lines {s}-{e} (old)"),
            (None, None) => "Line ?".to_string(),
        }
    }
}

/// Context for editing an existing annotation (set when user presses `e` in annotation menu).
#[derive(Debug, Clone)]
pub struct EditingAnnotation {
    pub file_path: String,
    pub old_range: Option<(u32, u32)>,
    pub new_range: Option<(u32, u32)>,
    pub old_comment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    DiffExplorer,
    WorktreeBrowser,
    AgentOutputs,
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
    pub commit_message: TextBuffer,
    pub target_dialog_open: bool,
    pub target_dialog_input: TextBuffer,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub target_label: String,
    pub hud_expanded: bool,

    // Visual selection
    pub selection: SelectionState,

    // Annotations
    pub annotations: AnnotationState,

    // Comment editor
    pub comment_editor_open: bool,
    pub comment_editor_text: TextBuffer,

    // Prompt preview
    pub prompt_preview_visible: bool,
    pub prompt_preview_text: String,

    // Annotation menu
    pub annotation_menu_open: bool,
    pub annotation_menu_items: Vec<AnnotationMenuItem>,
    pub annotation_menu_selected: usize,
    pub editing_annotation: Option<EditingAnnotation>,

    // Agent
    pub agent_outputs: AgentOutputsState,
    pub agent_selector: AgentSelectorState,

    // PTY focus mode
    pub pty_focus: bool,

    // Review state tracking
    pub review: ReviewState,

    // Restore confirm
    pub restore_confirm_open: bool,

    // Theme
    pub theme: Theme,

    // Settings modal
    pub settings: SettingsState,
}

impl AppState {
    pub fn new(diff_options: DiffOptions, theme: Theme) -> Self {
        Self {
            active_view: ActiveView::DiffExplorer,
            focus: FocusPanel::Navigator,
            diff: DiffState::new(diff_options),
            navigator: NavigatorState::new(),
            worktree: WorktreeState::new(),
            should_quit: false,
            commit_dialog_open: false,
            commit_message: TextBuffer::new(),
            target_dialog_open: false,
            target_dialog_input: TextBuffer::new(),
            status_message: None,
            target_label: String::new(),
            hud_expanded: false,
            selection: SelectionState::default(),
            annotations: AnnotationState::default(),
            comment_editor_open: false,
            comment_editor_text: TextBuffer::new(),
            prompt_preview_visible: false,
            prompt_preview_text: String::new(),
            annotation_menu_open: false,
            annotation_menu_items: Vec::new(),
            annotation_menu_selected: 0,
            editing_annotation: None,
            agent_outputs: AgentOutputsState::default(),
            agent_selector: AgentSelectorState::default(),
            pty_focus: false,
            review: ReviewState::default(),
            restore_confirm_open: false,
            theme,
            settings: SettingsState::default(),
        }
    }
}
