use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::text_input::render_text_input;
use crate::state::AppState;

pub fn render_commit_dialog(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 10.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Commit Message ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning));

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
        state.commit_message.text(),
        state.commit_message.cursor_char_index(),
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
        Span::styled("commit  ", Style::default().fg(theme.text_muted)),
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
