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
    #[allow(dead_code)]
    pub horizontal_scroll: usize,
    pub loading: bool,
    /// Per-line highlight spans for the old side, indexed by 1-based line number.
    pub old_highlights: Vec<Vec<HighlightSpan>>,
    /// Per-line highlight spans for the new side, indexed by 1-based line number.
    pub new_highlights: Vec<Vec<HighlightSpan>>,
}

impl DiffState {
    pub fn new(options: DiffOptions) -> Self {
        Self {
            options,
            deltas: Vec::new(),
            selected_file: None,
            scroll_offset: 0,
            horizontal_scroll: 0,
            loading: false,
            old_highlights: Vec::new(),
            new_highlights: Vec::new(),
        }
    }

    pub fn selected_delta(&self) -> Option<&FileDelta> {
        self.selected_file.and_then(|i| self.deltas.get(i))
    }

    pub fn total_lines(&self) -> usize {
        self.selected_delta()
            .map(|d| d.hunks.iter().map(|h| h.lines.len() + 1).sum::<usize>())
            .unwrap_or(0)
    }
}
