use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render a text input that wraps and scrolls within the given area.
/// Supports embedded newlines. The cursor (block character) is always visible.
pub fn render_text_input(frame: &mut Frame, area: Rect, text: &str, style: Style) {
    if area.width < 3 || area.height == 0 {
        return;
    }

    // Available width for text (1 char left padding, 1 char for cursor at end of line)
    let inner_width = (area.width as usize).saturating_sub(2);
    if inner_width == 0 {
        return;
    }

    // Split on real newlines first, then wrap each paragraph
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
        } else {
            let mut remaining = paragraph;
            loop {
                if remaining.len() <= inner_width {
                    lines.push(remaining.to_string());
                    break;
                }
                let (chunk, rest) = remaining.split_at(inner_width);
                lines.push(chunk.to_string());
                remaining = rest;
            }
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    // The cursor is on the last line (since we're always appending)
    let cursor_line = lines.len().saturating_sub(1);
    let visible_height = area.height as usize;

    // Scroll so cursor line is visible
    let scroll = if cursor_line >= visible_height {
        cursor_line - visible_height + 1
    } else {
        0
    };

    // Build display lines with scroll
    let display_lines: Vec<Line> = lines
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, line_text)| {
            if i == cursor_line {
                Line::from(Span::styled(format!(" {}\u{2588}", line_text), style))
            } else {
                Line::from(Span::styled(format!(" {}", line_text), style))
            }
        })
        .collect();

    let paragraph = Paragraph::new(display_lines);
    frame.render_widget(paragraph, area);
}
