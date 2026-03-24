use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_file_picker(frame: &mut Frame, state: &AppState) {
    if !state.file_picker.active {
        return;
    }

    let theme = &state.theme;
    let area = frame.area();

    let dialog_width = 70.min(area.width.saturating_sub(4)).max(40);
    let dialog_height = 20.min(area.height.saturating_sub(4)).max(8);

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" File Picker (Ctrl+P) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // search input
            Constraint::Length(1), // separator
            Constraint::Min(1),    // file list
            Constraint::Length(1), // hints
        ])
        .split(inner);

    // Search input with cursor
    let query = state.file_picker.query.text();
    let cursor_pos = state.file_picker.query.cursor_char_index();

    let mut input_spans = vec![Span::styled(
        " \u{1F50D} ",
        Style::default().fg(theme.text_muted),
    )];

    if query.is_empty() {
        input_spans.push(Span::styled(
            "Type to filter files...",
            Style::default().fg(theme.text_muted),
        ));
    } else {
        let chars: Vec<char> = query.chars().collect();
        let before: String = chars[..cursor_pos.min(chars.len())].iter().collect();
        let after: String = chars[cursor_pos.min(chars.len())..].iter().collect();

        input_spans.push(Span::styled(before, Style::default().fg(theme.text)));
        input_spans.push(Span::styled(
            "\u{2588}",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
        if !after.is_empty() {
            input_spans.push(Span::styled(after, Style::default().fg(theme.text)));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(input_spans)), rows[0]);

    // Separator
    let sep: String = "\u{2500}".repeat(inner.width as usize);
    frame.render_widget(
        Paragraph::new(sep).style(Style::default().fg(theme.text_muted)),
        rows[1],
    );

    // File list
    let list_height = rows[2].height as usize;
    let filtered = &state.file_picker.filtered;
    let selected = state.file_picker.selected;

    // Compute scroll offset to keep selected item visible
    let scroll = if selected >= list_height {
        selected - list_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (vis_idx, filt) in filtered.iter().enumerate().skip(scroll).take(list_height) {
        if let Some(entry) = state.file_picker.entries.get(filt.entry_index) {
            let is_selected = vis_idx == selected;
            let prefix = if is_selected { " \u{25b6} " } else { "   " };

            let stats = format!(" +{}/\u{2212}{}", entry.additions, entry.deletions);

            let max_path_len =
                (inner.width as usize).saturating_sub(prefix.len() + stats.len() + 1);

            let path_display = if entry.path.len() > max_path_len {
                let truncated = &entry.path[entry.path.len().saturating_sub(max_path_len)..];
                format!("\u{2026}{truncated}")
            } else {
                entry.path.clone()
            };

            if is_selected {
                // Build path with match highlights
                let path_spans =
                    build_highlighted_path(&path_display, &filt.match_indices, theme, true);

                let mut spans = vec![Span::styled(
                    prefix,
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )];
                spans.extend(path_spans);
                spans.push(Span::styled(stats, Style::default().fg(theme.warning)));
                lines.push(Line::from(spans));
            } else {
                let path_spans =
                    build_highlighted_path(&path_display, &filt.match_indices, theme, false);

                let mut spans = vec![Span::styled(prefix, Style::default().fg(theme.text_muted))];
                spans.extend(path_spans);
                spans.push(Span::styled(stats, Style::default().fg(theme.text_muted)));
                lines.push(Line::from(spans));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "   No matching files",
            Style::default().fg(theme.text_muted),
        )));
    }

    frame.render_widget(Paragraph::new(lines), rows[2]);

    // Hints
    let count_text = format!("{}/{}", filtered.len(), state.file_picker.entries.len());
    let hints = Line::from(vec![
        Span::styled(
            format!(" {count_text} "),
            Style::default().fg(theme.text_muted),
        ),
        Span::styled(
            "[\u{2191}/\u{2193}]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("navigate ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("select ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("close", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[3]);
}

use crate::theme::Theme;

fn build_highlighted_path(
    path: &str,
    match_indices: &[u32],
    theme: &Theme,
    is_selected: bool,
) -> Vec<Span<'static>> {
    let normal_style = if is_selected {
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    if match_indices.is_empty() {
        return vec![Span::styled(path.to_owned(), normal_style)];
    }

    let highlight_style = Style::default()
        .fg(theme.warning)
        .add_modifier(Modifier::BOLD);

    let chars: Vec<char> = path.chars().collect();
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut in_highlight = false;

    for (i, &ch) in chars.iter().enumerate() {
        let is_match = match_indices.contains(&(i as u32));
        if is_match != in_highlight {
            if !current.is_empty() {
                let style = if in_highlight {
                    highlight_style
                } else {
                    normal_style
                };
                spans.push(Span::styled(std::mem::take(&mut current), style));
            }
            in_highlight = is_match;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        let style = if in_highlight {
            highlight_style
        } else {
            normal_style
        };
        spans.push(Span::styled(current, style));
    }

    spans
}
