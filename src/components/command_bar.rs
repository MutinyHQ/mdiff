use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_command_bar(frame: &mut Frame, state: &AppState) {
    if !state.command_bar.active {
        return;
    }

    let theme = &state.theme;
    let area = frame.area();

    let bar_area = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
    frame.render_widget(Clear, bar_area);

    let query = state.command_bar.buffer.text();
    let cursor_pos = state.command_bar.buffer.cursor_char_index();

    let mut spans = vec![Span::styled(
        ":",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )];

    if query.is_empty() {
        spans.push(Span::styled(
            "▏",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
    } else {
        let chars: Vec<char> = query.chars().collect();
        let before: String = chars[..cursor_pos.min(chars.len())].iter().collect();
        let after: String = chars[cursor_pos.min(chars.len())..].iter().collect();

        spans.push(Span::styled(before, Style::default().fg(theme.text)));
        spans.push(Span::styled(
            "▏",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
        if !after.is_empty() {
            spans.push(Span::styled(after, Style::default().fg(theme.text)));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), bar_area);
}
