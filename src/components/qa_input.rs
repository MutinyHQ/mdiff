use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_qa_input(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();

    let bar_height = 3;
    if area.height < bar_height {
        return;
    }

    let bar_area = Rect::new(
        0,
        area.height.saturating_sub(bar_height),
        area.width,
        bar_height,
    );

    let block = Block::default()
        .title(" Ask a Question ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let inner = block.inner(bar_area);
    frame.render_widget(block, bar_area);

    let query = state.qa.input_buffer.text();
    let cursor_pos = state.qa.input_buffer.cursor_char_index();

    let mut spans = vec![Span::styled(
        "? ",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )];

    if query.is_empty() {
        spans.push(Span::styled(
            "Type your question...",
            Style::default().fg(theme.text_muted),
        ));
        spans.push(Span::styled(
            "▏",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
    } else {
        let before = &query[..cursor_pos.min(query.len())];
        let after = &query[cursor_pos.min(query.len())..];

        spans.push(Span::styled(before, Style::default().fg(theme.text)));
        spans.push(Span::styled(
            "▏",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
        spans.push(Span::styled(after, Style::default().fg(theme.text)));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), inner);
}
