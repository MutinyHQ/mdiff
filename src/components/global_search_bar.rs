use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_global_search_bar(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();

    // Position at the bottom of the screen
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
        .title(" Global Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let inner = block.inner(bar_area);
    frame.render_widget(block, bar_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // search input
            Constraint::Length(1), // match info
        ])
        .split(inner);

    // Search input line with cursor
    let query = state.global_search.query.text();
    let cursor_pos = state.global_search.query.cursor_char_index();

    let mut input_spans = vec![Span::styled(
        "Search: ",
        Style::default().fg(theme.text_muted),
    )];

    if query.is_empty() {
        input_spans.push(Span::styled("_", Style::default().fg(theme.text_muted)));
    } else {
        let before = &query[..cursor_pos.min(query.len())];
        let after = &query[cursor_pos.min(query.len())..];

        input_spans.push(Span::styled(before, Style::default().fg(theme.text)));
        input_spans.push(Span::styled(
            "▏",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
        input_spans.push(Span::styled(after, Style::default().fg(theme.text)));
    }

    frame.render_widget(Paragraph::new(Line::from(input_spans)), rows[0]);

    // Match count and current file info
    let match_info = if state.global_search.matches.is_empty() {
        if query.is_empty() {
            Line::from(vec![
                Span::styled(
                    "[n]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("next  ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[N]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("prev  ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[Esc]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("close", Style::default().fg(theme.text_muted)),
            ])
        } else {
            Line::from(Span::styled(
                "No matches found",
                Style::default().fg(theme.error),
            ))
        }
    } else {
        let current = state.global_search.current_match + 1;
        let total = state.global_search.matches.len();
        let current_match = &state.global_search.matches[state.global_search.current_match];

        Line::from(vec![
            Span::styled(
                format!("Match {}/{} in ", current, total),
                Style::default().fg(theme.text),
            ),
            Span::styled(
                &current_match.file_path,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" (line {})", current_match.line_number),
                Style::default().fg(theme.text_muted),
            ),
        ])
    };

    frame.render_widget(Paragraph::new(match_info), rows[1]);
}
