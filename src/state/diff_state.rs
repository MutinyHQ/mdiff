use std::collections::HashMap;

use crate::git::types::FileDelta;
use crate::highlight::HighlightSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Split,
    Unified,
}

#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub ignore_whitespace: bool,
    pub view_mode: DiffViewMode,
}

impl DiffOptions {
    pub fn new(ignore_whitespace: bool, unified: bool) -> Self {
        Self {
            ignore_whitespace,
            view_mode: if unified {
                DiffViewMode::Unified
            } else {
                DiffViewMode::Split
            },
        }
    }
}

pub struct DiffState {
    pub options: DiffOptions,
    pub deltas: Vec<FileDelta>,
    pub selected_file: Option<usize>,
    pub scroll_offset: usize,
    pub cursor_row: usize,
    pub viewport_height: usize,
    pub loading: bool,
    /// Per-line highlight spans for the old side, indexed by 1-based line number.
    pub old_highlights: Vec<Vec<HighlightSpan>>,
    /// Per-line highlight spans for the new side, indexed by 1-based line number.
    pub new_highlights: Vec<Vec<HighlightSpan>>,
    /// Number of context lines to show around each change (default 3).
    pub display_context: usize,
    /// Per-gap expansion state: gap_id -> extra lines revealed.
    pub gap_expansions: HashMap<usize, usize>,

    // Diff text search
    pub search_active: bool,
    pub search_query: String,
    /// Display row indices that match the search query.
    pub search_matches: Vec<usize>,
    /// Current position within `search_matches`.
    pub search_match_index: Option<usize>,
}

impl DiffState {
    pub fn new(options: DiffOptions) -> Self {
        Self {
            options,
            deltas: Vec::new(),
            selected_file: None,
            scroll_offset: 0,
            cursor_row: 0,
            viewport_height: 20,
            loading: false,
            old_highlights: Vec::new(),
            new_highlights: Vec::new(),
            display_context: 3,
            gap_expansions: HashMap::new(),
            search_active: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_index: None,
        }
    }

    pub fn selected_delta(&self) -> Option<&FileDelta> {
        self.selected_file.and_then(|i| self.deltas.get(i))
    }
}
