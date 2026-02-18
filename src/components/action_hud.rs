use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::app_state::ActiveView;
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

        let bindings: &[(&str, &str)] = if state.active_view == ActiveView::AgentOutputs {
            &[
                ("j/k", "select"),
                ("J/K", "scroll"),
                ("y", "copy"),
                ("^A", "re-run"),
                ("^K", "kill"),
                ("Esc", "back"),
            ]
        } else if state.selection.active {
            &[
                ("j/k", "extend"),
                ("i", "comment"),
                ("d", "delete"),
                ("y", "yank"),
                ("v/Esc", "exit"),
                ("]", "next"),
                ("[", "prev"),
            ]
        } else {
            &[
                ("q", "quit"),
                ("j/k", "nav"),
                ("/", "search"),
                ("Tab", "view"),
                ("w", "ws"),

                ("s", "stage"),
                ("u", "unstage"),
                ("r", "restore"),
                ("c", "commit"),
                ("v", "visual"),
                ("a", "annotate"),
                ("y", "yank"),
                ("Space", "expand"),
                ("p", "preview"),
                ("g", "refresh"),
                ("t", "target"),
                ("o", "outputs"),
                ("^W", "worktree"),
                ("^A", "agent"),
            ]
        };

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

        // Show annotation count on the right side
        let ann_count = state.annotations.count();
        if ann_count > 0 {
            // Calculate space needed for right-aligned text
            let ann_text = format!(" {ann_count} annotations ");
            let used_width: usize = spans.iter().map(|s| s.width()).sum();
            let remaining = (area.width as usize).saturating_sub(used_width + ann_text.len());
            if remaining > 0 {
                spans.push(Span::raw(" ".repeat(remaining)));
                spans.push(Span::styled(
                    ann_text,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        let bar =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));
        frame.render_widget(bar, area);
    }
}
