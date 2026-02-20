use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render a text input that wraps and scrolls within the given area.
/// Supports embedded newlines. The cursor (block character) is shown at `cursor_char_index`.
pub fn render_text_input(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    cursor_char_index: usize,
    style: Style,
) {
    if area.width < 3 || area.height == 0 {
        return;
    }

    // Available width for text (1 char left padding, 1 char for cursor at end of line)
    let inner_width = (area.width as usize).saturating_sub(2);
    if inner_width == 0 {
        return;
    }

    // Split on real newlines first, then wrap each paragraph.
    // Track whether each visual line ends with a real newline or is a wrap continuation.
    let mut lines: Vec<String> = Vec::new();
    let mut is_newline_after: Vec<bool> = Vec::new(); // true if a real newline follows this line

    let paragraphs: Vec<&str> = text.split('\n').collect();
    for (pi, paragraph) in paragraphs.iter().enumerate() {
        if paragraph.is_empty() {
            lines.push(String::new());
            is_newline_after.push(pi + 1 < paragraphs.len());
        } else {
            let chars: Vec<char> = paragraph.chars().collect();
            let mut pos = 0;
            while pos < chars.len() {
                let end = (pos + inner_width).min(chars.len());
                let chunk: String = chars[pos..end].iter().collect();
                lines.push(chunk);
                pos = end;
                // Mark whether this visual line ends with a real newline or is a wrap
                if pos >= chars.len() {
                    is_newline_after.push(pi + 1 < paragraphs.len());
                } else {
                    is_newline_after.push(false); // wrap continuation
                }
            }
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
        is_newline_after.push(false);
    }

    // Find which visual line and column the cursor falls on
    let mut chars_remaining = cursor_char_index;
    let mut cursor_line = lines.len().saturating_sub(1);
    let mut cursor_col = 0;
    for (i, line_text) in lines.iter().enumerate() {
        let line_chars = line_text.chars().count();
        if chars_remaining <= line_chars {
            cursor_line = i;
            cursor_col = chars_remaining;
            break;
        }
        chars_remaining -= line_chars;
        // Only consume a char for the newline if this visual line ends at a real newline
        if is_newline_after[i] {
            if chars_remaining > 0 {
                chars_remaining -= 1;
            } else {
                cursor_line = i;
                cursor_col = line_chars;
                break;
            }
        }
    }
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
                // Insert block cursor at the right column
                let before: String = line_text.chars().take(cursor_col).collect();
                let after: String = line_text.chars().skip(cursor_col).collect();
                Line::from(Span::styled(format!(" {}\u{2588}{}", before, after), style))
            } else {
                Line::from(Span::styled(format!(" {}", line_text), style))
            }
        })
        .collect();

    let paragraph = Paragraph::new(display_lines);
    frame.render_widget(paragraph, area);
}
