use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;

/// Render the prompt preview pane showing the rendered template.
pub fn render_prompt_preview(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" Prompt Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    if state.prompt_preview_text.is_empty() {
        let msg = Paragraph::new(" Select lines and press [y] to generate a prompt")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let lines: Vec<Line> = state
        .prompt_preview_text
        .lines()
        .map(|l| {
            let style = if l.starts_with('+') {
                Style::default().fg(Color::Green)
            } else if l.starts_with('-') {
                Style::default().fg(Color::Red)
            } else if l.starts_with("File:") || l.starts_with("Instruction:") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(format!(" {l}"), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
