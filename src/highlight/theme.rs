use ratatui::style::{Modifier, Style};

use crate::theme::SyntaxColors;

/// All recognized highlight capture names, in order.
/// The index into this array corresponds to the Highlight ID returned by tree-sitter.
pub const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "escape",
    "function",
    "function.builtin",
    "function.method",
    "keyword",
    "label",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

pub fn highlight_names_vec() -> Vec<String> {
    HIGHLIGHT_NAMES.iter().map(|s| s.to_string()).collect()
}

/// Map a highlight index to a ratatui Style using theme syntax colors.
pub fn style_for_highlight(idx: usize, syntax: &SyntaxColors) -> Style {
    let name = HIGHLIGHT_NAMES.get(idx).copied().unwrap_or("");
    match name {
        "comment" => Style::default().fg(syntax.comment),
        "keyword" => Style::default()
            .fg(syntax.keyword)
            .add_modifier(Modifier::BOLD),
        "string" | "string.special" => Style::default().fg(syntax.string),
        "number" | "constant" | "constant.builtin" => Style::default().fg(syntax.number),
        "function" | "function.builtin" | "function.method" => {
            Style::default().fg(syntax.function)
        }
        "type" | "type.builtin" | "constructor" => Style::default().fg(syntax.type_name),
        "variable.builtin" => Style::default().fg(syntax.property),
        "variable" | "variable.parameter" => Style::default().fg(syntax.variable),
        "operator" => Style::default().fg(syntax.operator),
        "property" | "label" => Style::default().fg(syntax.property),
        "attribute" => Style::default().fg(syntax.number),
        "tag" => Style::default().fg(syntax.tag),
        "escape" => Style::default().fg(syntax.operator),
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" | "punctuation.special" => {
            Style::default().fg(syntax.punctuation)
        }
        _ => Style::default().fg(syntax.default_fg),
    }
}
