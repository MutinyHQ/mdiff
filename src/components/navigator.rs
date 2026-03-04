use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::review_state::FileReviewStatus;
use crate::state::{app_state::FocusPanel, AppState};

use super::Component;

pub struct Navigator;

impl Component for Navigator {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focus == FocusPanel::Navigator;
        let theme = &state.theme;

        let border_style = if is_focused {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let visible = state.navigator.visible_entries();
        let total = visible.len();

        let title = if state.navigator.search_active {
            let q = state.navigator.search_query.text();
            let ci = state.navigator.search_query.cursor_char_index();
            let before: String = q.chars().take(ci).collect();
            let after: String = q.chars().skip(ci).collect();
            format!(" /{}\u{2588}{} ", before, after)
        } else {
            format!(" Files ({total}) ")
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        if visible.is_empty() {
            let msg = if state.navigator.search_active {
                " No matches"
            } else {
                " No changes"
            };
            let paragraph = Paragraph::new(msg)
                .style(Style::default().fg(theme.text_muted))
                .block(block);
            frame.render_widget(paragraph, area);
            return;
        }

        let inner_height = area.height.saturating_sub(2) as usize;
        // Available width for the display text after prefix ("▶ ") and review icon ("✓ ")
        let inner_width = area.width.saturating_sub(2) as usize; // block borders
        let prefix_width = 5; // "▶ " (3) + "✓ " (2, icon is 1 char + space)
        let max_display_width = inner_width.saturating_sub(prefix_width);
        let selected = state.navigator.selected;

        let scroll = if selected >= inner_height {
            selected - inner_height + 1
        } else {
            0
        };

        let lines: Vec<Line> = visible
            .iter()
            .enumerate()
            .skip(scroll)
            .take(inner_height)
            .map(|(vis_idx, (_entry_idx, entry))| {
                let is_selected = vis_idx == selected;
                let is_active = state.diff.selected_file == Some(entry.delta_index);

                let style = if is_selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                        .bg(theme.selection_bg)
                } else if is_active {
                    Style::default()
                        .fg(theme.text)
                        .bg(theme.selection_inactive_bg)
                } else {
                    Style::default().fg(theme.text)
                };

                let prefix = if is_selected { "\u{25b6}" } else { " " };

                // Review status icon
                let review_status = state.review.status(&entry.path);
                let (review_icon, review_color) = match review_status {
                    FileReviewStatus::Reviewed => ("\u{2713}", theme.success), // ✓
                    FileReviewStatus::Unreviewed => ("\u{25cb}", theme.text_muted), // ○
                    FileReviewStatus::ChangedSinceReview => ("\u{25cf}", theme.warning), // ●
                    FileReviewStatus::New => ("\u{2605}", theme.accent),       // ★
                };

                let display = middle_ellipsis(&entry.display, max_display_width);

                Line::from(vec![
                    Span::styled(format!("{prefix} "), style),
                    Span::styled(format!("{review_icon} "), Style::default().fg(review_color)),
                    Span::styled(display, style),
                ])
            })
            .collect();

        let scroll_info = if total > inner_height {
            format!(" {}/{} ", selected + 1, total)
        } else {
            String::new()
        };

        let block = block.title_bottom(Line::from(scroll_info).right_aligned());
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }
}

fn middle_ellipsis(s: &str, max_chars: usize) -> String {
    let len = s.chars().count();
    if len <= max_chars {
        return s.to_string();
    }
    match max_chars {
        0 => String::new(),
        1 => "\u{2026}".to_string(),
        2 => {
            let first = s.chars().next().unwrap_or('\u{2026}');
            format!("{first}\u{2026}")
        }
        _ => {
            let keep = max_chars - 1;
            let head = keep / 2;
            let tail = keep - head; // Bias one extra char to the tail when odd.
            let start: String = s.chars().take(head).collect();
            let end: String = s.chars().skip(len - tail).collect();
            format!("{start}\u{2026}{end}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::middle_ellipsis;

    #[test]
    fn returns_original_when_short_enough() {
        assert_eq!(middle_ellipsis("abc", 3), "abc");
        assert_eq!(middle_ellipsis("abc", 10), "abc");
    }

    #[test]
    fn handles_small_width_edge_cases() {
        assert_eq!(middle_ellipsis("abcdef", 0), "");
        assert_eq!(middle_ellipsis("abcdef", 1), "…");
        assert_eq!(middle_ellipsis("abcdef", 2), "a…");
        assert_eq!(middle_ellipsis("abcdef", 3), "a…f");
    }

    #[test]
    fn truncates_with_middle_ellipsis_and_tail_bias() {
        let out = middle_ellipsis("src/components/navigator.rs [M] +12 -4", 20);
        assert_eq!(out.chars().count(), 20);
        assert!(out.starts_with("src/compo"));
        assert!(out.ends_with("[M] +12 -4"));
    }
}
