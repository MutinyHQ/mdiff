use std::collections::HashMap;
use std::path::Path;

use ratatui::style::Style;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use super::languages::{detect_language, language_entries};
use super::theme::{highlight_names_vec, style_for_highlight};

/// A span of styled text within a line.
#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

pub struct HighlightEngine {
    configs: HashMap<String, HighlightConfiguration>,
    highlight_names: Vec<String>,
}

impl HighlightEngine {
    pub fn new() -> Self {
        let highlight_names = highlight_names_vec();
        let mut configs = HashMap::new();

        for entry in language_entries() {
            let config = entry.config(&highlight_names);
            configs.insert(entry.name.to_string(), config);
        }

        Self {
            configs,
            highlight_names,
        }
    }

    /// Highlight a file's content and return per-line highlight spans.
    /// Returns None if the language is not recognized.
    pub fn highlight_lines(&self, path: &Path, content: &str) -> Option<Vec<Vec<HighlightSpan>>> {
        let lang_name = detect_language(path)?;
        let config = self.configs.get(lang_name)?;

        let mut highlighter = Highlighter::new();
        let events = highlighter
            .highlight(config, content.as_bytes(), None, |_| None)
            .ok()?;

        let lines: Vec<&str> = content.split('\n').collect();
        let mut result: Vec<Vec<HighlightSpan>> = vec![Vec::new(); lines.len()];

        let mut current_style = Style::default();

        for event in events {
            match event.ok()? {
                HighlightEvent::Source { start, end } => {
                    // Map byte range to line(s)
                    add_spans_for_range(&lines, &mut result, start, end, current_style);
                }
                HighlightEvent::HighlightStart(highlight) => {
                    current_style = style_for_highlight(highlight.0);
                }
                HighlightEvent::HighlightEnd => {
                    current_style = Style::default();
                }
            }
        }

        Some(result)
    }
}

/// Add highlight spans across line boundaries for a byte range.
fn add_spans_for_range(
    lines: &[&str],
    result: &mut [Vec<HighlightSpan>],
    start: usize,
    end: usize,
    style: Style,
) {
    if start >= end {
        return;
    }

    // Find which line(s) this range belongs to
    let mut line_start_byte = 0;
    for (line_idx, line) in lines.iter().enumerate() {
        let line_end_byte = line_start_byte + line.len();

        // Check if this range overlaps with this line
        if start < line_end_byte + 1 && end > line_start_byte {
            let span_start = start.saturating_sub(line_start_byte).min(line.len());
            let span_end = (end - line_start_byte).min(line.len());

            if span_start < span_end && line_idx < result.len() {
                result[line_idx].push(HighlightSpan {
                    start: span_start,
                    end: span_end,
                    style,
                });
            }
        }

        if line_start_byte > end {
            break;
        }

        // +1 for the newline character
        line_start_byte = line_end_byte + 1;
    }
}
