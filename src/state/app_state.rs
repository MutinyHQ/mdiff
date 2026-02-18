use crate::theme::Theme;

use super::{
    AgentOutputsState, AgentSelectorState, AnnotationState, DiffOptions, DiffState, NavigatorState,
    ReviewState, SelectionState, WorktreeState,
};

use super::settings_state::SettingsState;

/// Snapshot of an annotation for the annotation menu (owned to avoid borrow issues).
#[derive(Debug, Clone)]
pub struct AnnotationMenuItem {
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub comment: String,
}

/// Context for editing an existing annotation (set when user presses `e` in annotation menu).
#[derive(Debug, Clone)]
pub struct EditingAnnotation {
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
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
    pub commit_message: String,
    pub target_dialog_open: bool,
    pub target_dialog_input: String,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub target_label: String,
    pub hud_expanded: bool,

    // Visual selection
    pub selection: SelectionState,

    // Annotations
    pub annotations: AnnotationState,

    // Comment editor
    pub comment_editor_open: bool,
    pub comment_editor_text: String,

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
            commit_message: String::new(),
            target_dialog_open: false,
            target_dialog_input: String::new(),
            status_message: None,
            target_label: String::new(),
            hud_expanded: false,
            selection: SelectionState::default(),
            annotations: AnnotationState::default(),
            comment_editor_open: false,
            comment_editor_text: String::new(),
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
