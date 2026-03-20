use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::navigator_state::{TreeNode, TreeNodeKind};
use crate::state::review_state::FileReviewStatus;
use crate::state::{app_state::FocusPanel, AppState};

use super::Component;

pub struct Navigator;

impl Component for Navigator {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        if state.navigator.tree_mode && !state.navigator.search_active {
            render_tree_mode(frame, area, state);
        } else {
            render_flat_mode(frame, area, state);
        }
    }
}

fn render_flat_mode(frame: &mut Frame, area: Rect, state: &AppState) {
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
    let inner_width = area.width.saturating_sub(2) as usize;
    let prefix_width = 5;
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

            let review_status = state.review.status(&entry.path);
            let (review_icon, review_color) = match review_status {
                FileReviewStatus::Reviewed => ("\u{2713}", theme.success),
                FileReviewStatus::Unreviewed => ("\u{25cb}", theme.text_muted),
                FileReviewStatus::ChangedSinceReview => ("\u{25cf}", theme.warning),
                FileReviewStatus::New => ("\u{2605}", theme.accent),
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

fn render_tree_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focus == FocusPanel::Navigator;
    let theme = &state.theme;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let tree_nodes = state.navigator.visible_tree_nodes();
    let total = tree_nodes.len();

    let title = format!(" Tree ({total}) ");
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    if tree_nodes.is_empty() {
        let paragraph = Paragraph::new(" No changes")
            .style(Style::default().fg(theme.text_muted))
            .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let inner_width = area.width.saturating_sub(2) as usize;
    let selected = state.navigator.tree_selected;

    let scroll = if selected >= inner_height {
        selected - inner_height + 1
    } else {
        0
    };

    let lines: Vec<Line> = tree_nodes
        .iter()
        .enumerate()
        .skip(scroll)
        .take(inner_height)
        .map(|(idx, node)| render_tree_node(node, idx, selected, inner_width, state))
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

fn render_tree_node<'a>(
    node: &TreeNode,
    idx: usize,
    selected: usize,
    max_width: usize,
    state: &'a AppState,
) -> Line<'a> {
    let theme = &state.theme;
    let is_selected = idx == selected;

    let base_style = if is_selected {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
            .bg(theme.selection_bg)
    } else {
        Style::default().fg(theme.text)
    };

    match &node.kind {
        TreeNodeKind::Directory {
            name,
            collapsed,
            file_count,
            total_additions,
            total_deletions,
        } => {
            let arrow = if *collapsed { "\u{25b6}" } else { "\u{25bc}" };
            let indent = build_indent(node.depth);

            let reviewed_count = count_reviewed_in_dir(node, state);
            let stats = if *collapsed {
                format!(
                    " ({reviewed}/{file_count} reviewed)",
                    reviewed = reviewed_count
                )
            } else {
                format!(" ({file_count} files, +{total_additions} -{total_deletions})",)
            };

            let dir_style = if is_selected {
                base_style
            } else {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            };

            let stats_style = if is_selected {
                base_style
            } else {
                Style::default().fg(theme.text_muted)
            };

            let display_name = format!("{name}/");
            let total_prefix_len = indent.chars().count() + 2 + display_name.chars().count();
            let remaining = max_width.saturating_sub(total_prefix_len);
            let stats_display = middle_ellipsis(&stats, remaining);

            Line::from(vec![
                Span::styled(indent, base_style),
                Span::styled(format!("{arrow} "), dir_style),
                Span::styled(display_name, dir_style),
                Span::styled(stats_display, stats_style),
            ])
        }
        TreeNodeKind::File { entry_index } => {
            let entry = &state.navigator.entries[*entry_index];
            let is_active = state.diff.selected_file == Some(entry.delta_index);

            let style = if is_selected {
                base_style
            } else if is_active {
                Style::default()
                    .fg(theme.text)
                    .bg(theme.selection_inactive_bg)
            } else {
                Style::default().fg(theme.text)
            };

            let indent = build_indent(node.depth);
            let guide = build_tree_guide(node, idx, state);

            let review_status = state.review.status(&entry.path);
            let (review_icon, review_color) = match review_status {
                FileReviewStatus::Reviewed => ("\u{2713}", theme.success),
                FileReviewStatus::Unreviewed => ("\u{25cb}", theme.text_muted),
                FileReviewStatus::ChangedSinceReview => ("\u{25cf}", theme.warning),
                FileReviewStatus::New => ("\u{2605}", theme.accent),
            };

            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);

            let file_stats = extract_file_stats(&entry.display);

            let prefix_len = indent.chars().count()
                + guide.chars().count()
                + 2 // review icon + space
                + filename.chars().count();
            let remaining = max_width.saturating_sub(prefix_len + 1);
            let stats_display = middle_ellipsis(&file_stats, remaining);

            Line::from(vec![
                Span::styled(indent, style),
                Span::styled(guide, Style::default().fg(theme.text_muted)),
                Span::styled(format!("{review_icon} "), Style::default().fg(review_color)),
                Span::styled(filename.to_string(), style),
                Span::styled(
                    format!(" {stats_display}"),
                    if is_selected {
                        style
                    } else {
                        Style::default().fg(theme.text_muted)
                    },
                ),
            ])
        }
    }
}

fn build_indent(depth: usize) -> String {
    if depth == 0 {
        return String::new();
    }
    "  ".repeat(depth.saturating_sub(1))
}

fn build_tree_guide(node: &TreeNode, idx: usize, state: &AppState) -> String {
    if node.depth == 0 {
        return String::new();
    }
    let tree_nodes = state.navigator.visible_tree_nodes();
    let is_last = is_last_sibling(idx, tree_nodes);
    if is_last {
        "\u{2514}\u{2500} ".to_string() // └─
    } else {
        "\u{251c}\u{2500} ".to_string() // ├─
    }
}

fn is_last_sibling(idx: usize, nodes: &[TreeNode]) -> bool {
    let node = &nodes[idx];
    let depth = node.depth;
    if depth == 0 {
        return false;
    }
    for next in nodes.iter().skip(idx + 1) {
        if next.depth < depth {
            return true;
        }
        if next.depth == depth {
            return false;
        }
    }
    true
}

fn count_reviewed_in_dir(dir_node: &TreeNode, state: &AppState) -> usize {
    let dir_path = &dir_node.path;
    state
        .navigator
        .entries
        .iter()
        .filter(|e| {
            let parts: Vec<&str> = e.path.rsplitn(2, '/').collect();
            let file_dir = if parts.len() == 2 { parts[1] } else { "" };
            file_dir == dir_path || file_dir.starts_with(&format!("{dir_path}/"))
        })
        .filter(|e| matches!(state.review.status(&e.path), FileReviewStatus::Reviewed))
        .count()
}

fn extract_file_stats(display: &str) -> String {
    let mut parts = Vec::new();
    for token in display.split_whitespace() {
        if (token.starts_with('[') && token.ends_with(']'))
            || token.starts_with('+')
            || token.starts_with('-')
        {
            parts.push(token.to_string());
        }
    }
    parts.join(" ")
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
