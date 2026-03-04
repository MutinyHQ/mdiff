use super::TextBuffer;

/// Global search state for searching across all diff content.
#[derive(Debug, Clone)]
pub struct GlobalSearchState {
    pub active: bool,
    pub query: TextBuffer,
    pub matches: Vec<GlobalSearchMatch>,
    pub current_match: usize,
}

/// A single match in the global search results.
#[derive(Debug, Clone)]
pub struct GlobalSearchMatch {
    pub file_index: usize,
    pub file_path: String,
    pub line_number: u32,
    /// The display row within the file's diff view (for scrolling)
    pub display_row: usize,
}

impl Default for GlobalSearchState {
    fn default() -> Self {
        Self {
            active: false,
            query: TextBuffer::new(),
            matches: Vec::new(),
            current_match: 0,
        }
    }
}
