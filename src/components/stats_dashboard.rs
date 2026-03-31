use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::git::types::FileStatus;
use crate::state::AppState;

use super::Component;

pub struct StatsDashboard;

impl Component for StatsDashboard {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;
        let block = Block::default()
            .title(" Diff Statistics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let summary = match &state.stats.diff_summary {
            Some(s) => s,
            None => {
                let empty = Paragraph::new("  No changes to display")
                    .style(Style::default().fg(theme.text_muted));
                frame.render_widget(empty, inner);
                return;
            }
        };

        if summary.total_files == 0 {
            let empty = Paragraph::new("  No changes to display")
                .style(Style::default().fg(theme.text_muted));
            frame.render_widget(empty, inner);
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        // Header: aggregate stats
        lines.push(Line::from(""));

        let mut status_parts: Vec<String> = Vec::new();
        if summary.files_added > 0 {
            status_parts.push(format!("{} added", summary.files_added));
        }
        if summary.files_modified > 0 {
            status_parts.push(format!("{} modified", summary.files_modified));
        }
        if summary.files_deleted > 0 {
            status_parts.push(format!("{} deleted", summary.files_deleted));
        }
        if summary.files_renamed > 0 {
            status_parts.push(format!("{} renamed", summary.files_renamed));
        }
        let status_text = if status_parts.is_empty() {
            String::new()
        } else {
            format!(" ({})", status_parts.join(", "))
        };

        lines.push(Line::from(vec![
            Span::styled("  Files: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{} changed", summary.total_files),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(status_text, Style::default().fg(theme.text_muted)),
        ]));

        let net = summary.total_additions as i64 - summary.total_deletions as i64;
        let net_str = if net >= 0 {
            format!("net +{}", net)
        } else {
            format!("net {}", net)
        };
        lines.push(Line::from(vec![
            Span::styled("  Lines: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("+{}", format_number(summary.total_additions)),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" "),
            Span::styled(
                format!("-{}", format_number(summary.total_deletions)),
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                format!("  ({})", net_str),
                Style::default().fg(theme.text_muted),
            ),
        ]));

        // File type breakdown
        if !summary.by_extension.is_empty() {
            let ext_spans: Vec<Span> = summary
                .by_extension
                .iter()
                .take(8)
                .enumerate()
                .flat_map(|(i, (ext, count))| {
                    let mut spans = Vec::new();
                    if i > 0 {
                        spans.push(Span::raw("  "));
                    }
                    spans.push(Span::styled(
                        format!("{ext} ({count})"),
                        Style::default().fg(ext_color(i)),
                    ));
                    spans
                })
                .collect();

            let mut type_line = vec![Span::styled(
                "  Types: ",
                Style::default().fg(theme.text_muted),
            )];
            type_line.extend(ext_spans);
            lines.push(Line::from(type_line));
        }

        lines.push(Line::from(""));

        // Sort indicator
        let sort_label = state.stats.sort_mode.label();
        lines.push(Line::from(vec![
            Span::styled(
                format!("  Sort: {} ▾", sort_label),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [s] cycle sort", Style::default().fg(theme.text_muted)),
        ]));

        // Separator
        let sep_width = inner.width.saturating_sub(2) as usize;
        lines.push(Line::from(Span::styled(
            format!("  {}", "─".repeat(sep_width.saturating_sub(2))),
            Style::default().fg(theme.text_muted),
        )));

        // File list
        let sorted_stats = summary.sorted_file_stats(state.stats.sort_mode);
        let max_churn = sorted_stats
            .iter()
            .map(|f| f.churn())
            .max()
            .unwrap_or(1)
            .max(1);

        let bar_width = 10usize;
        let avail_width = inner.width as usize;

        for (i, file) in sorted_stats.iter().enumerate() {
            let is_selected = i == state.stats.selected_index;

            let marker = if is_selected { " ▸ " } else { "   " };

            let status_badge = match file.status {
                FileStatus::Added => "[A] ",
                FileStatus::Deleted => "[D] ",
                FileStatus::Renamed => "[R→] ",
                FileStatus::Untracked => "[?] ",
                FileStatus::Modified => "",
            };

            let display_path = truncate_path(&file.path, avail_width.saturating_sub(40));

            let additions_str = if file.binary {
                "binary".to_string()
            } else {
                format!("+{}", file.additions)
            };
            let deletions_str = if file.binary {
                String::new()
            } else {
                format!("-{}", file.deletions)
            };

            let churn = file.churn();
            let filled = if max_churn > 0 {
                churn * bar_width / max_churn
            } else {
                0
            };
            let add_portion = if churn > 0 {
                file.additions * filled / churn.max(1)
            } else {
                0
            };
            let del_portion = filled.saturating_sub(add_portion);
            let empty = bar_width.saturating_sub(filled);

            let add_bar = "█".repeat(add_portion);
            let del_bar = "█".repeat(del_portion);
            let empty_bar = "░".repeat(empty);

            let highlight_style = if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let status_color = match file.status {
                FileStatus::Added => Color::Green,
                FileStatus::Deleted => Color::Red,
                FileStatus::Renamed => Color::Yellow,
                _ => theme.text_muted,
            };

            let mut spans = vec![
                Span::styled(marker, highlight_style),
                Span::styled(status_badge.to_string(), Style::default().fg(status_color)),
                Span::styled(display_path, highlight_style),
            ];

            let padding = "  ";
            spans.push(Span::raw(padding));
            spans.push(Span::styled(
                format!("{:<6}", additions_str),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::styled(
                format!("{:<6}", deletions_str),
                Style::default().fg(Color::Red),
            ));
            spans.push(Span::styled(add_bar, Style::default().fg(Color::Green)));
            spans.push(Span::styled(del_bar, Style::default().fg(Color::Red)));
            spans.push(Span::styled(
                empty_bar,
                Style::default().fg(theme.text_muted),
            ));

            lines.push(Line::from(spans));
        }

        lines.push(Line::from(""));

        // Footer with key bindings
        lines.push(Line::from(vec![
            Span::styled(
                "  [j/k]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" navigate  "),
            Span::styled(
                "[Enter]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" jump to file  "),
            Span::styled(
                "[s]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" sort  "),
            Span::styled("[Esc]", Style::default().fg(theme.text_muted)),
            Span::raw(" close"),
        ]));

        let visible_lines: Vec<Line> = lines.into_iter().skip(state.stats.scroll_offset).collect();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!(
            "{},{:03},{:03}",
            n / 1_000_000,
            (n / 1_000) % 1_000,
            n % 1_000
        )
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1_000, n % 1_000)
    } else {
        n.to_string()
    }
}

fn ext_color(index: usize) -> Color {
    const COLORS: &[Color] = &[
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Magenta,
        Color::Blue,
        Color::Red,
        Color::LightCyan,
        Color::LightYellow,
    ];
    COLORS[index % COLORS.len()]
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len || max_len < 4 {
        return path.to_string();
    }
    let take = max_len.saturating_sub(3);
    format!("…{}", &path[path.len() - take..])
}
