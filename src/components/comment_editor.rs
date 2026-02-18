use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::text_input::render_text_input;
use crate::state::AppState;

pub fn render_comment_editor(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 12.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let (sel_start, sel_end) = state.selection.range();
    let title = format!(" Comment (lines {}-{}) ", sel_start + 1, sel_end + 1);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // text area (expands)
            Constraint::Length(1), // blank
            Constraint::Length(1), // hints
        ])
        .split(inner);

    // Text input with wrapping and scroll
    render_text_input(
        frame,
        rows[0],
        &state.comment_editor_text,
        Style::default().fg(theme.text),
    );

    // Hints
    let hints = Line::from(vec![
        Span::styled(
            " [Enter]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("save  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[S-Enter]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("newline  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("cancel", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[2]);
}
