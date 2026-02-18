use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;

/// Render the prompt preview pane showing the rendered template.
pub fn render_prompt_preview(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let block = Block::default()
        .title(" Prompt Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary));

    if state.prompt_preview_text.is_empty() {
        let msg = Paragraph::new(" Select lines and press [y] to generate a prompt")
            .style(Style::default().fg(theme.text_muted))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let lines: Vec<Line> = state
        .prompt_preview_text
        .lines()
        .map(|l| {
            let style = if l.starts_with('+') {
                Style::default().fg(theme.diff_add_fg)
            } else if l.starts_with('-') {
                Style::default().fg(theme.diff_del_fg)
            } else if l.starts_with("File:") || l.starts_with("Instruction:") {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.text)
            };
            Line::from(Span::styled(format!(" {l}"), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
