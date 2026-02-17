use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::AppState;

use super::Component;

pub struct ActionHud;

impl Component for ActionHud {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Show status message if present, otherwise show keybindings
        if let Some((ref msg, is_error)) = state.status_message {
            let color = if is_error { Color::Red } else { Color::Green };
            let bar = Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(msg.as_str(), Style::default().fg(color)),
            ]))
            .style(Style::default().bg(Color::Rgb(30, 30, 30)));
            frame.render_widget(bar, area);
            return;
        }

        let bindings = [
            ("q", "quit"),
            ("j/k", "nav"),
            ("/", "search"),
            ("Tab", "view"),
            ("w", "ws"),
            ("s", "stage"),
            ("u", "unstage"),
            ("r", "restore"),
            ("c", "commit"),
        ];

        let mut spans = Vec::new();
        spans.push(Span::raw(" "));
        for (i, (key, desc)) in bindings.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", Style::default().fg(Color::DarkGray)));
            }
            spans.push(Span::styled(
                format!("[{key}]"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                (*desc).to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let bar =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));
        frame.render_widget(bar, area);
    }
}
