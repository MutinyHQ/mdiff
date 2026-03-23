use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_bookmark_list(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.bookmarks.list_visible || state.bookmarks.bookmarks.is_empty() {
        return;
    }

    let sorted = state.bookmarks.all_sorted();
    let count = sorted.len();

    let panel_width = 38u16.min(area.width.saturating_sub(2));
    let panel_height = (count as u16 + 2).min(area.height.saturating_sub(2)).max(4);

    let x = area.x + area.width.saturating_sub(panel_width + 1);
    let y = area.y + 1;
    let overlay_area = Rect::new(x, y, panel_width, panel_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(format!(" Bookmarks ({count}) "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.accent));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let max_visible = inner.height as usize;
    let selected = state.bookmarks.list_selected;

    let scroll_offset = if selected >= max_visible {
        selected - max_visible + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (display_idx, (_, bm)) in sorted
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_visible)
    {
        let is_selected = display_idx == selected;

        let diamond = if let Some(c) = bm.label {
            format!("\u{25c6}{c}")
        } else {
            "\u{25c6} ".to_string()
        };

        let location = bm.location();
        let max_loc_width = inner.width as usize - 4;
        let truncated = if location.len() > max_loc_width {
            let start = location.len() - max_loc_width + 1;
            format!("\u{2026}{}", &location[start..])
        } else {
            location
        };

        let style = if is_selected {
            Style::default()
                .fg(state.theme.text)
                .bg(state.theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.text)
        };

        let diamond_style = if is_selected {
            Style::default()
                .fg(state.theme.accent)
                .bg(state.theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.accent)
        };

        lines.push(Line::from(vec![
            Span::styled(diamond, diamond_style),
            Span::styled(format!(" {truncated}"), style),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
