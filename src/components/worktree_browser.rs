use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

use super::Component;

pub struct WorktreeBrowser;

impl Component for WorktreeBrowser {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;

        let block = Block::default()
            .title(" Worktree Browser ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        if state.worktree.worktrees.is_empty() {
            let msg = if state.worktree.loading {
                " Loading worktrees..."
            } else {
                " No worktrees found"
            };
            let paragraph = Paragraph::new(msg)
                .style(Style::default().fg(theme.text_muted))
                .block(block);
            frame.render_widget(paragraph, area);
            return;
        }

        let inner_height = area.height.saturating_sub(2) as usize;
        let selected = state.worktree.selected;

        let scroll = if selected >= inner_height {
            selected - inner_height + 1
        } else {
            0
        };

        let lines: Vec<Line> = state
            .worktree
            .worktrees
            .iter()
            .enumerate()
            .skip(scroll)
            .take(inner_height)
            .map(|(idx, wt)| {
                let is_selected = idx == selected;

                let row_style = if is_selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                        .bg(theme.selection_bg)
                } else {
                    Style::default().fg(theme.text)
                };

                let prefix = if is_selected { "\u{25b6}" } else { " " };

                // Status indicator
                let status_span = if wt.is_dirty {
                    Span::styled("\u{25cf} ", Style::default().fg(theme.error))
                } else {
                    Span::styled("\u{25cf} ", Style::default().fg(theme.success))
                };

                // Name
                let name_style = if wt.is_main {
                    row_style.add_modifier(Modifier::BOLD)
                } else {
                    row_style
                };
                let name_span = Span::styled(format!("{:<16}", wt.name), name_style);

                // Path (abbreviated)
                let path_str = abbreviate_path(&wt.path);
                let path_span = Span::styled(
                    format!("{:<30}", path_str),
                    Style::default().fg(theme.text_muted),
                );

                // Branch
                let branch = wt.head_ref.as_deref().unwrap_or("(detached)");
                let branch_span = Span::styled(
                    format!("{:<20}", branch),
                    Style::default().fg(theme.warning),
                );

                // Agent badge
                let agent_span = if let Some(ref agent) = wt.agent {
                    Span::styled(
                        format!("[{}]", agent.agent_type.label()),
                        Style::default()
                            .fg(theme.secondary)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::raw("")
                };

                Line::from(vec![
                    Span::styled(format!("{prefix} "), row_style),
                    status_span,
                    name_span,
                    path_span,
                    branch_span,
                    agent_span,
                ])
            })
            .collect();

        let total = state.worktree.worktrees.len();
        let scroll_info = if total > inner_height {
            format!(" {}/{} ", selected + 1, total)
        } else {
            String::new()
        };

        let block = block.title_bottom(Line::from(vec![
            Span::styled(
                " [Enter]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("switch  ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[f]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("freeze  ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[r]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("refresh  ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "[Esc]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("back ", Style::default().fg(theme.text_muted)),
            Span::styled(scroll_info, Style::default().fg(theme.text_muted)),
        ]));

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }
}

fn abbreviate_path(path: &std::path::Path) -> String {
    if let Some(home) = dirs_next_home() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

fn dirs_next_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}
