use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::app_state::FocusPanel;
use crate::state::auto_review_state::FindingStatus;
use crate::state::AppState;

use super::Component;

pub struct AutoReviewPanel;

impl Component for AutoReviewPanel {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;
        let ar = &state.auto_review;
        let focused = state.focus == FocusPanel::ReviewPanel;

        let (reviewed, total) = ar.reviewed_count();
        let filter_label = ar.filter.label();
        let title = format!(" Findings {reviewed}/{total} [{filter_label}] ");

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if focused {
                theme.accent
            } else {
                theme.text_muted
            }));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 3 || ar.findings.is_empty() {
            if ar.findings.is_empty() {
                let msg =
                    Paragraph::new(" No findings").style(Style::default().fg(theme.text_muted));
                frame.render_widget(msg, inner);
            }
            return;
        }

        let detail_height = 6u16;
        let hints_height = 1u16;
        let list_height = inner
            .height
            .saturating_sub(detail_height + hints_height + 1);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(list_height.max(3)),
                Constraint::Length(1), // separator
                Constraint::Length(detail_height),
                Constraint::Length(hints_height),
            ])
            .split(inner);

        let filtered = ar.filtered_indices();

        // Determine scroll for the list
        let selected_pos = filtered.iter().position(|&i| i == ar.selected).unwrap_or(0);
        let visible_rows = list_height as usize;
        let scroll = if selected_pos >= ar.scroll_offset + visible_rows {
            selected_pos.saturating_sub(visible_rows - 1)
        } else if selected_pos < ar.scroll_offset {
            selected_pos
        } else {
            ar.scroll_offset
        };

        // Render findings list
        let mut lines: Vec<Line> = Vec::new();
        let mut current_file: Option<&str> = None;

        for &idx in filtered.iter().skip(scroll).take(visible_rows) {
            let finding = &ar.findings[idx];
            let is_selected = idx == ar.selected;

            // File header when file changes
            if current_file != Some(&finding.file_path) {
                current_file = Some(&finding.file_path);
                if !lines.is_empty() && lines.len() < visible_rows {
                    lines.push(Line::from(""));
                }
                if lines.len() < visible_rows {
                    lines.push(Line::from(Span::styled(
                        format!(" {}", finding.file_path),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    )));
                }
            }

            if lines.len() >= visible_rows {
                break;
            }

            let status_icon = finding.status.icon();
            let severity_color = severity_color(finding.severity, theme);
            let severity_label = finding.severity.label();
            let category_label = finding.category.label();
            let range = finding.range_text();

            let dimmed = finding.status == FindingStatus::Dismissed;
            let text_style = if dimmed {
                Style::default().fg(theme.text_muted)
            } else if is_selected {
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let prefix = if is_selected && focused { "▸" } else { " " };

            let truncated_comment: String = {
                let max_len = (inner.width as usize).saturating_sub(30);
                let first_line = finding.comment.lines().next().unwrap_or("");
                if first_line.len() > max_len {
                    format!("{}…", &first_line[..max_len.saturating_sub(1)])
                } else {
                    first_line.to_string()
                }
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{prefix}{status_icon} "),
                    Style::default().fg(if dimmed {
                        theme.text_muted
                    } else {
                        match finding.status {
                            FindingStatus::Pending => theme.warning,
                            FindingStatus::Accepted => Color::Green,
                            FindingStatus::Dismissed => theme.text_muted,
                        }
                    }),
                ),
                Span::styled(
                    severity_label.to_string(),
                    Style::default().fg(severity_color),
                ),
                Span::styled(
                    format!(" {category_label}"),
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(format!(" {range} "), Style::default().fg(theme.text_muted)),
                Span::styled(truncated_comment, text_style),
            ]));
        }

        let list_widget = Paragraph::new(lines);
        frame.render_widget(list_widget, layout[0]);

        // Separator
        let sep = Paragraph::new(Line::from("─".repeat(inner.width as usize)))
            .style(Style::default().fg(theme.text_muted));
        frame.render_widget(sep, layout[1]);

        // Detail area for selected finding
        if let Some(finding) = ar.findings.get(ar.selected) {
            let detail_lines: Vec<Line> = vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} ", finding.severity.label()),
                        Style::default().fg(severity_color(finding.severity, theme)),
                    ),
                    Span::styled(
                        format!("[{}] ", finding.category.label()),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        finding.status.label(),
                        Style::default().fg(match finding.status {
                            FindingStatus::Pending => theme.warning,
                            FindingStatus::Accepted => Color::Green,
                            FindingStatus::Dismissed => theme.text_muted,
                        }),
                    ),
                ]),
                Line::from(Span::styled(
                    format!("{} {}", finding.file_path, finding.range_text()),
                    Style::default().fg(theme.text_muted),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    &finding.comment,
                    Style::default().fg(if finding.status == FindingStatus::Dismissed {
                        theme.text_muted
                    } else {
                        theme.text
                    }),
                )),
            ];

            let detail = Paragraph::new(detail_lines).wrap(Wrap { trim: false });
            frame.render_widget(detail, layout[2]);
        }

        // Hints
        let hints = Line::from(vec![
            Span::styled(
                " [j/k]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" nav ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[Enter]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" jump ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[a]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" accept ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[x]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" dismiss ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[e]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" edit ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[f]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" filter", Style::default().fg(theme.text_muted)),
        ]);
        frame.render_widget(Paragraph::new(hints), layout[3]);
    }
}

fn severity_color(
    severity: crate::state::annotation_state::AnnotationSeverity,
    theme: &crate::theme::Theme,
) -> Color {
    use crate::state::annotation_state::AnnotationSeverity;
    match severity {
        AnnotationSeverity::Critical => Color::Red,
        AnnotationSeverity::Major => theme.warning,
        AnnotationSeverity::Minor => Color::Yellow,
        AnnotationSeverity::Info => Color::Cyan,
    }
}
