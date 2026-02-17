use ratatui::style::{Color, Modifier, Style};

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

/// Map a highlight index to a ratatui Style.
pub fn style_for_highlight(idx: usize) -> Style {
    let name = HIGHLIGHT_NAMES.get(idx).copied().unwrap_or("");
    match name {
        "comment" => Style::default().fg(Color::Rgb(106, 115, 125)),
        "keyword" => Style::default()
            .fg(Color::Rgb(198, 120, 221))
            .add_modifier(Modifier::BOLD),
        "string" | "string.special" => Style::default().fg(Color::Rgb(152, 195, 121)),
        "number" | "constant" | "constant.builtin" => {
            Style::default().fg(Color::Rgb(209, 154, 102))
        }
        "function" | "function.builtin" | "function.method" => {
            Style::default().fg(Color::Rgb(97, 175, 239))
        }
        "type" | "type.builtin" | "constructor" => Style::default().fg(Color::Rgb(229, 192, 123)),
        "variable.builtin" => Style::default().fg(Color::Rgb(224, 108, 117)),
        "variable" | "variable.parameter" => Style::default().fg(Color::Rgb(171, 178, 191)),
        "operator" => Style::default().fg(Color::Rgb(86, 182, 194)),
        "property" | "label" => Style::default().fg(Color::Rgb(224, 108, 117)),
        "attribute" => Style::default().fg(Color::Rgb(209, 154, 102)),
        "tag" => Style::default().fg(Color::Rgb(224, 108, 117)),
        "escape" => Style::default().fg(Color::Rgb(86, 182, 194)),
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" | "punctuation.special" => {
            Style::default().fg(Color::Rgb(140, 140, 140))
        }
        _ => Style::default().fg(Color::Rgb(171, 178, 191)),
    }
}
