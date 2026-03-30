use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;

use super::Component;

pub struct QAAnswerPanel;

impl Component for QAAnswerPanel {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;
        let qa = &state.qa;

        let history_label = match qa.history_display() {
            Some((cur, total)) => format!(" Q&A ({}/{}) ", cur, total),
            None => " Q&A ".to_string(),
        };

        let streaming = qa.answer_streaming;
        let title = if streaming {
            format!("{}\u{25cf}", history_label.trim_end())
        } else {
            history_label
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if streaming {
                theme.warning
            } else {
                theme.accent
            }));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 3 || inner.width < 4 {
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // question
                Constraint::Min(1),    // answer
                Constraint::Length(1), // hints
            ])
            .split(inner);

        // Question display
        let question_text = qa.current_question().unwrap_or("");
        let q_line = Line::from(vec![
            Span::styled(
                "Q: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(question_text, Style::default().fg(theme.text)),
        ]);
        let q_widget = Paragraph::new(q_line).wrap(Wrap { trim: false });
        frame.render_widget(q_widget, layout[0]);

        // Answer display with scrolling
        let answer = &qa.answer_text;
        let lines: Vec<Line> = if answer.is_empty() && streaming {
            vec![Line::from(Span::styled(
                "Thinking...",
                Style::default().fg(theme.text_muted),
            ))]
        } else if answer.is_empty() {
            vec![Line::from(Span::styled(
                "(no answer yet)",
                Style::default().fg(theme.text_muted),
            ))]
        } else {
            answer.lines().map(|l| Line::from(l.to_string())).collect()
        };

        let total_lines = lines.len() as u16;
        let visible_height = layout[1].height;
        let max_scroll = total_lines.saturating_sub(visible_height) as usize;
        let scroll = qa.answer_scroll.min(max_scroll);

        let answer_widget = Paragraph::new(lines)
            .style(Style::default().fg(theme.text))
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0));
        frame.render_widget(answer_widget, layout[1]);

        // Hints
        let hints = if streaming {
            Line::from(vec![Span::styled(
                " Streaming...",
                Style::default().fg(theme.text_muted),
            )])
        } else {
            Line::from(vec![
                Span::styled(
                    " [j/k]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" scroll ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[Esc]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" close ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[^]/^[]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" history", Style::default().fg(theme.text_muted)),
            ])
        };
        frame.render_widget(Paragraph::new(hints), layout[2]);
    }
}
