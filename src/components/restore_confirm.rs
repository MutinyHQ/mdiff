use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_restore_confirm(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();
    let dialog_width = 50.min(area.width.saturating_sub(4));
    let dialog_height = 7.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Confirm Restore ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.error));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // warning text
            Constraint::Length(1), // file path
            Constraint::Length(1), // blank
            Constraint::Length(1), // key hints
        ])
        .split(inner);

    // Warning
    let warning = Line::from(vec![Span::styled(
        " This will discard all unstaged changes to:",
        Style::default().fg(theme.warning),
    )]);
    frame.render_widget(Paragraph::new(warning), rows[0]);

    // File path
    let file_name = state
        .diff
        .selected_file
        .and_then(|idx| state.diff.deltas.get(idx))
        .map(|d| format!(" {}", d.path.display()))
        .unwrap_or_default();
    let path_line = Line::from(vec![Span::styled(
        file_name,
        Style::default()
            .fg(theme.text)
            .add_modifier(Modifier::BOLD),
    )]);
    frame.render_widget(Paragraph::new(path_line), rows[1]);

    // Key hints
    let hints = Line::from(vec![
        Span::styled(
            " [Enter/y]",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("confirm  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Esc/n]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("cancel", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[3]);
}
