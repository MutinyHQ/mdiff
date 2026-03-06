use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;

use super::Component;

pub struct FeedbackSummary;

impl Component for FeedbackSummary {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;
        let block = Block::default()
            .title(" Feedback Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        let total_ann = state.annotations.count();
        let total_scores = state.annotations.score_count();
        let reviewed = state.review.reviewed_count();
        let total_files = state.navigator.entries.len();

        if total_ann == 0 && total_scores == 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  No feedback yet",
                Style::default()
                    .fg(theme.text_muted)
                    .add_modifier(Modifier::ITALIC),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Press [v] in the diff view to select lines and [i] to add comments.",
                Style::default().fg(theme.text_muted),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} annotations", total_ann),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" · "),
                Span::styled(
                    format!("{} scores", total_scores),
                    Style::default().fg(theme.accent),
                ),
                Span::raw(" · "),
                Span::styled(
                    format!("{}/{} files reviewed", reviewed, total_files),
                    Style::default().fg(theme.text_muted),
                ),
            ]));
            lines.push(Line::from(""));

            if total_scores > 0 {
                lines.push(Line::from(Span::styled(
                    "  Score Distribution",
                    Style::default().add_modifier(Modifier::BOLD).fg(theme.text),
                )));
                lines.push(Line::from(""));

                let all_scores = state.annotations.all_scores_sorted();
                let mut dist = [0usize; 5];
                let mut sum = 0usize;
                for s in &all_scores {
                    if s.score >= 1 && s.score <= 5 {
                        dist[(s.score - 1) as usize] += 1;
                        sum += s.score as usize;
                    }
                }

                let max_count = *dist.iter().max().unwrap_or(&1).max(&1);
                let bar_width = 20usize;
                let colors = [
                    Color::Red,
                    Color::Rgb(255, 165, 0),
                    Color::Yellow,
                    Color::Rgb(144, 238, 144),
                    Color::Green,
                ];

                for (i, count) in dist.iter().enumerate() {
                    let filled = if max_count > 0 {
                        count * bar_width / max_count
                    } else {
                        0
                    };
                    let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                    let pct = if total_scores > 0 {
                        *count * 100 / total_scores
                    } else {
                        0
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", i + 1), Style::default().fg(colors[i])),
                        Span::styled("● ", Style::default().fg(colors[i])),
                        Span::styled(bar, Style::default().fg(colors[i])),
                        Span::styled(
                            format!("  {} ({}%)", count, pct),
                            Style::default().fg(theme.text_muted),
                        ),
                    ]));
                }

                let avg = if total_scores > 0 {
                    sum as f64 / total_scores as f64
                } else {
                    0.0
                };
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("  Average: {:.1}/5", avg),
                    Style::default().add_modifier(Modifier::BOLD).fg(theme.text),
                )));
                lines.push(Line::from(""));
            }

            if total_ann > 0 {
                lines.push(Line::from(Span::styled(
                    "  Per-File Annotation Density",
                    Style::default().add_modifier(Modifier::BOLD).fg(theme.text),
                )));
                lines.push(Line::from(""));

                let mut file_stats: Vec<(String, usize)> = state
                    .annotations
                    .annotations
                    .iter()
                    .map(|(path, anns)| (path.clone(), anns.len()))
                    .collect();
                file_stats.sort_by(|a, b| b.1.cmp(&a.1));

                for (file_path, count) in file_stats.iter().take(10) {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {:40}", file_path),
                            Style::default().fg(theme.text),
                        ),
                        Span::styled(
                            format!("{:>3} annotations", count),
                            Style::default().fg(theme.text_muted),
                        ),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "  [y]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Copy JSON  "),
            Span::styled(
                "[p]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Copy prompt text  "),
            Span::styled("[Esc]", Style::default().fg(theme.text_muted)),
            Span::raw(" Close"),
        ]));

        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(state.feedback_summary_scroll)
            .collect();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }
}
