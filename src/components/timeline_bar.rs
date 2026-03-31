use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::AppState;
use crate::theme::Theme;

pub fn render_timeline_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.timeline.active || state.timeline.commits.is_empty() {
        return;
    }

    let theme = &state.theme;
    let commits = &state.timeline.commits;
    let selected = state.timeline.selected_index;

    let mut spans: Vec<Span> = Vec::new();

    // "ALL" marker
    let all_style = if selected.is_none() {
        Style::default()
            .fg(theme.surface)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };
    spans.push(Span::styled(" ALL ", all_style));
    spans.push(Span::raw(" "));

    // Commit markers
    let max_markers = (area.width as usize).saturating_sub(8) / 4;
    let display_count = commits.len().min(max_markers);

    for (i, _commit) in commits.iter().enumerate().take(display_count) {
        let marker_style = if Some(i) == selected {
            Style::default()
                .fg(theme.surface)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let label = format!("{}", i + 1);
        spans.push(Span::styled(format!(" {label} "), marker_style));

        if i + 1 < display_count {
            spans.push(Span::styled("─", Style::default().fg(theme.text_muted)));
        }
    }

    if commits.len() > max_markers {
        spans.push(Span::styled(
            format!(" +{}", commits.len() - max_markers),
            Style::default().fg(theme.text_muted),
        ));
    }

    // Info about selected commit
    let info = match selected {
        None => format!("  {} commits total", commits.len()),
        Some(i) => {
            let c = &commits[i];
            let short_oid = if c.oid.len() >= 7 {
                &c.oid[..7]
            } else {
                &c.oid
            };
            format!(
                "  {} {} (+{}/-{} {}) {} by {}",
                short_oid,
                truncate_str(&c.summary, 40),
                c.additions,
                c.deletions,
                if c.files_changed == 1 {
                    "1 file".to_string()
                } else {
                    format!("{} files", c.files_changed)
                },
                c.timestamp,
                c.author,
            )
        }
    };
    spans.push(Span::styled(info, Style::default().fg(theme.text)));

    let line = Line::from(spans);
    let bar = Paragraph::new(line).style(Style::default().bg(bar_bg(theme)));
    frame.render_widget(bar, area);
}

fn bar_bg(theme: &Theme) -> ratatui::style::Color {
    theme.surface
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}
