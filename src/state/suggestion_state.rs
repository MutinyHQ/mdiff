use crate::state::text_buffer::TextBuffer;

#[derive(Debug, Default)]
pub struct SuggestionState {
    /// Whether the suggestion editor is currently open.
    pub active: bool,
    /// The original code being replaced (read-only display).
    pub original_code: Vec<String>,
    /// Editable buffer for the replacement code.
    pub replacement: TextBuffer,
    /// Optional comment explaining the suggestion.
    pub comment: TextBuffer,
    /// Whether the preview pane is visible.
    pub preview_visible: bool,
    /// Which pane is focused: Comment, Replacement.
    pub focus: SuggestionFocus,
    /// File path for the anchor.
    pub file_path: String,
    /// Old-file line range.
    pub old_range: Option<(u32, u32)>,
    /// New-file line range.
    pub new_range: Option<(u32, u32)>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SuggestionFocus {
    Comment,
    #[default]
    Replacement,
}

impl SuggestionState {
    pub fn reset(&mut self) {
        self.active = false;
        self.original_code.clear();
        self.replacement.clear();
        self.comment.clear();
        self.preview_visible = false;
        self.focus = SuggestionFocus::default();
        self.file_path.clear();
        self.old_range = None;
        self.new_range = None;
    }
}
