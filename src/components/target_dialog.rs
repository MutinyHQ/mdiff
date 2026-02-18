use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render_target_dialog(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();
    // Center a dialog box
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 9.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Compare Against ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.success));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // current target
            Constraint::Length(1), // blank
            Constraint::Length(1), // input line
            Constraint::Length(1), // blank
            Constraint::Length(1), // hint text
            Constraint::Length(1), // key hints
        ])
        .split(inner);

    // Current target display
    let current = Line::from(vec![
        Span::styled(" current: ", Style::default().fg(theme.text_muted)),
        Span::styled(
            &state.target_label,
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(current), rows[0]);

    // Input line with cursor
    let input_text = format!(" {}\u{2588}", &state.target_dialog_input);
    let input = Paragraph::new(input_text).style(Style::default().fg(theme.text));
    frame.render_widget(input, rows[2]);

    // Hint
    let hint = Paragraph::new(Line::from(vec![Span::styled(
        " branch, tag, commit, or empty for HEAD",
        Style::default().fg(theme.text_muted),
    )]));
    frame.render_widget(hint, rows[4]);

    // Key hints
    let hints = Line::from(vec![
        Span::styled(
            " [Enter]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("confirm  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("cancel", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[5]);
}
