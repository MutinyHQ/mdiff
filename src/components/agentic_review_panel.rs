use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::state::app_state::FocusPanel;
use crate::state::AppState;

use super::text_input::render_text_input;
use super::Component;

pub struct AgenticReviewPanel;

impl Component for AgenticReviewPanel {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;
        let running = state.agentic_review_running;
        let composing = state.agentic_review_composing;
        let focused = state.focus == FocusPanel::ReviewPanel;
        let has_output = !state.agentic_review_stream_output.is_empty();

        let title = if running {
            " AI Review \u{25cf} "
        } else if composing {
            " AI Review \u{270e} " // pencil indicates composing
        } else {
            " AI Review "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if focused {
                theme.accent
            } else if running {
                theme.warning
            } else {
                theme.text_muted
            }));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 3 {
            return;
        }

        // Determine layout based on state
        let has_children = state.agentic_review_child_total > 0;
        let has_draft = !state.agentic_review_text.text().is_empty();
        let show_input = (composing || has_draft) && !running;

        let mut constraints = Vec::new();

        if show_input {
            constraints.push(Constraint::Length(1)); // instructions
            constraints.push(Constraint::Min(3)); // text input area
        }

        if has_output || running {
            if show_input {
                // Split space between input and output
                constraints.pop(); // remove Min(3) for input
                constraints.push(Constraint::Length(4)); // fixed input area
                constraints.push(Constraint::Min(3)); // output area
            } else {
                constraints.push(Constraint::Min(3)); // output area
            }
        } else if !show_input {
            constraints.push(Constraint::Min(1)); // empty spacer
        }

        if has_children {
            constraints.push(Constraint::Length(3)); // progress bar
        }

        constraints.push(Constraint::Length(1)); // hints

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut idx = 0;

        // Render text input area when composing
        if show_input {
            // Instructions line
            let instr_style = Style::default().fg(theme.text_muted);
            let instr = if focused {
                Paragraph::new(" Type your review feedback:").style(instr_style)
            } else {
                Paragraph::new(" Focus with ^W j to type review:").style(instr_style)
            };
            frame.render_widget(instr, layout[idx]);
            idx += 1;

            // Text input
            if focused {
                render_text_input(
                    frame,
                    layout[idx],
                    state.agentic_review_text.text(),
                    state.agentic_review_text.cursor_char_index(),
                    Style::default().fg(theme.text),
                );
            } else {
                // Show text but no cursor when unfocused
                let text = state.agentic_review_text.text();
                let display = if text.is_empty() { "(empty)" } else { text };
                let p = Paragraph::new(display.to_string())
                    .style(Style::default().fg(theme.text_muted))
                    .wrap(Wrap { trim: false });
                frame.render_widget(p, layout[idx]);
            }
            idx += 1;
        }

        // Render stream output
        if has_output || running {
            let output = &state.agentic_review_stream_output;
            let lines: Vec<Line> = output.lines().map(|l| Line::from(l.to_string())).collect();
            let total_lines = lines.len() as u16;
            let visible_height = layout[idx].height;
            let max_scroll = total_lines.saturating_sub(visible_height) as usize;

            let scroll = if state.agentic_review_auto_scroll {
                max_scroll
            } else {
                state.agentic_review_scroll.min(max_scroll)
            };

            let output_widget = Paragraph::new(lines)
                .style(Style::default().fg(theme.text))
                .wrap(Wrap { trim: false })
                .scroll((scroll as u16, 0));
            frame.render_widget(output_widget, layout[idx]);
            idx += 1;
        } else if !show_input {
            idx += 1; // skip spacer
        }

        // Progress bar
        if has_children {
            let done = state.agentic_review_child_done;
            let total = state.agentic_review_child_total;
            let pct = if total > 0 {
                ((done as f64 / total as f64) * 100.0) as u16
            } else {
                0
            };

            let label = format!("{}/{} chunks", done, total);
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(if done == total {
                    Color::Green
                } else {
                    theme.accent
                }))
                .percent(pct.min(100))
                .label(label);
            frame.render_widget(gauge, layout[idx]);
            idx += 1;
        }

        // Hints
        let hints = if running {
            Line::from(vec![
                Span::styled(
                    " [Esc]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" cancel", Style::default().fg(theme.text_muted)),
            ])
        } else if composing && focused {
            Line::from(vec![
                Span::styled(
                    " [Enter]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" submit ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[S-Enter]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" newline ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[^W w]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" switch", Style::default().fg(theme.text_muted)),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    " [{/}]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" scroll ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[:review]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" dismiss ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    "[^R]",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" new", Style::default().fg(theme.text_muted)),
            ])
        };
        frame.render_widget(Paragraph::new(hints), layout[idx]);
    }
}
