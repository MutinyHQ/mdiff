use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::app_state::{ActiveView, FocusPanel};
use crate::state::AppState;

struct KeyEntry {
    key: &'static str,
    description: &'static str,
}

pub fn render_which_key(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.which_key_visible {
        return;
    }

    let entries = get_context_entries(state);
    if entries.is_empty() {
        return;
    }

    let max_key_width = entries.iter().map(|e| e.key.len()).max().unwrap_or(3);
    let max_desc_width = entries
        .iter()
        .map(|e| e.description.len())
        .max()
        .unwrap_or(10);
    let entry_width = max_key_width + max_desc_width + 3;

    let (cols, rows) = if entries.len() > 10 {
        (2, entries.len().div_ceil(2))
    } else {
        (1, entries.len())
    };

    let panel_width = ((entry_width * cols + 4).min(area.width as usize) as u16).max(30);
    let panel_height = ((rows + 2).min(area.height as usize) as u16).max(5);

    let x = area.x + area.width.saturating_sub(panel_width + 1);
    let y = area.y + area.height.saturating_sub(panel_height + 1);
    let overlay_area = Rect::new(x, y, panel_width, panel_height);

    frame.render_widget(Clear, overlay_area);

    let title = get_context_title(state);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.accent));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let mut lines: Vec<Line> = Vec::new();

    if cols == 2 {
        let half = entries.len().div_ceil(2);
        for i in 0..half {
            let mut spans = Vec::new();

            spans.push(Span::styled(
                format!("{:>width$}", entries[i].key, width = max_key_width),
                Style::default()
                    .fg(state.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(
                    "  {:<width$}",
                    entries[i].description,
                    width = max_desc_width
                ),
                Style::default().fg(state.theme.text),
            ));

            if i + half < entries.len() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("{:>width$}", entries[i + half].key, width = max_key_width),
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    format!("  {}", entries[i + half].description),
                    Style::default().fg(state.theme.text),
                ));
            }

            lines.push(Line::from(spans));
        }
    } else {
        for entry in &entries {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>width$}", entry.key, width = max_key_width),
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", entry.description),
                    Style::default().fg(state.theme.text),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn get_context_title(state: &AppState) -> &'static str {
    if state.selection.active {
        return "Visual Mode";
    }
    match state.active_view {
        ActiveView::WorktreeBrowser => "Worktree Browser",
        ActiveView::AgentOutputs => "Agent Outputs",
        ActiveView::FeedbackSummary => "Feedback Summary",
        ActiveView::DiffExplorer => match state.focus {
            FocusPanel::Navigator => "Navigator",
            FocusPanel::DiffView => "Diff View",
        },
    }
}

fn get_context_entries(state: &AppState) -> Vec<KeyEntry> {
    if state.selection.active {
        return vec![
            KeyEntry {
                key: "j/k",
                description: "Extend selection",
            },
            KeyEntry {
                key: "i",
                description: "Add annotation",
            },
            KeyEntry {
                key: "d",
                description: "Delete annotation",
            },
            KeyEntry {
                key: "y",
                description: "Copy prompt",
            },
            KeyEntry {
                key: "1-5",
                description: "Quick score",
            },
            KeyEntry {
                key: "v/Esc",
                description: "Exit visual",
            },
        ];
    }

    match state.active_view {
        ActiveView::WorktreeBrowser => vec![
            KeyEntry {
                key: "j/k",
                description: "Navigate",
            },
            KeyEntry {
                key: "Enter",
                description: "Select worktree",
            },
            KeyEntry {
                key: "r",
                description: "Refresh",
            },
            KeyEntry {
                key: "f",
                description: "Freeze",
            },
            KeyEntry {
                key: "Esc",
                description: "Back",
            },
        ],
        ActiveView::AgentOutputs => vec![
            KeyEntry {
                key: "j/k",
                description: "Navigate",
            },
            KeyEntry {
                key: "y",
                description: "Copy prompt",
            },
            KeyEntry {
                key: "w",
                description: "Switch worktree",
            },
            KeyEntry {
                key: "Enter",
                description: "PTY focus",
            },
            KeyEntry {
                key: "Ctrl+K",
                description: "Kill agent",
            },
            KeyEntry {
                key: "Esc",
                description: "Back",
            },
        ],
        ActiveView::FeedbackSummary => vec![
            KeyEntry {
                key: "j/k",
                description: "Scroll",
            },
            KeyEntry {
                key: "y",
                description: "Copy JSON",
            },
            KeyEntry {
                key: "p",
                description: "Copy prompt",
            },
            KeyEntry {
                key: "Esc/F",
                description: "Back to diff",
            },
        ],
        ActiveView::DiffExplorer => match state.focus {
            FocusPanel::Navigator => vec![
                KeyEntry {
                    key: "j/k",
                    description: "Navigate files",
                },
                KeyEntry {
                    key: "g/G",
                    description: "Top/bottom",
                },
                KeyEntry {
                    key: "l/Enter",
                    description: "Focus diff",
                },
                KeyEntry {
                    key: "/",
                    description: "Search files",
                },
                KeyEntry {
                    key: "m",
                    description: "Mark reviewed",
                },
                KeyEntry {
                    key: "n",
                    description: "Next unreviewed",
                },
                KeyEntry {
                    key: "s",
                    description: "Stage file",
                },
                KeyEntry {
                    key: "u",
                    description: "Unstage file",
                },
                KeyEntry {
                    key: "r",
                    description: "Restore file",
                },
                KeyEntry {
                    key: "c",
                    description: "Commit",
                },
                KeyEntry {
                    key: "t",
                    description: "Change target",
                },
                KeyEntry {
                    key: "o",
                    description: "Agent outputs",
                },
                KeyEntry {
                    key: "Ctrl+W",
                    description: "Worktrees",
                },
                KeyEntry {
                    key: "Ctrl+A",
                    description: "Agent selector",
                },
                KeyEntry {
                    key: "Ctrl+E",
                    description: "Export feedback",
                },
                KeyEntry {
                    key: "Tab",
                    description: "Split/unified",
                },
                KeyEntry {
                    key: "R",
                    description: "Refresh",
                },
                KeyEntry {
                    key: ":",
                    description: "Settings",
                },
                KeyEntry {
                    key: "?",
                    description: "This help",
                },
                KeyEntry {
                    key: "q",
                    description: "Quit",
                },
            ],
            FocusPanel::DiffView => vec![
                KeyEntry {
                    key: "j/k",
                    description: "Scroll",
                },
                KeyEntry {
                    key: "g/G",
                    description: "Top/bottom",
                },
                KeyEntry {
                    key: "h",
                    description: "Focus navigator",
                },
                KeyEntry {
                    key: "PgUp/Dn",
                    description: "Page scroll",
                },
                KeyEntry {
                    key: "Space",
                    description: "Expand context",
                },
                KeyEntry {
                    key: "/",
                    description: "Search in diff",
                },
                KeyEntry {
                    key: "n/N",
                    description: "Next/prev match",
                },
                KeyEntry {
                    key: "v",
                    description: "Visual select",
                },
                KeyEntry {
                    key: "i",
                    description: "Add annotation",
                },
                KeyEntry {
                    key: "a",
                    description: "Annotation menu",
                },
                KeyEntry {
                    key: "]",
                    description: "Next annotation",
                },
                KeyEntry {
                    key: "[",
                    description: "Prev annotation",
                },
                KeyEntry {
                    key: "p",
                    description: "Prompt preview",
                },
                KeyEntry {
                    key: "y",
                    description: "Copy prompt",
                },
                KeyEntry {
                    key: "1-5",
                    description: "Quick score",
                },
                KeyEntry {
                    key: "0",
                    description: "Remove score",
                },
                KeyEntry {
                    key: "s",
                    description: "Stage file",
                },
                KeyEntry {
                    key: "u",
                    description: "Unstage file",
                },
                KeyEntry {
                    key: "w",
                    description: "Toggle whitespace",
                },
                KeyEntry {
                    key: "Tab",
                    description: "Split/unified",
                },
                KeyEntry {
                    key: "Ctrl+E",
                    description: "Export feedback",
                },
                KeyEntry {
                    key: "?",
                    description: "This help",
                },
                KeyEntry {
                    key: "q",
                    description: "Quit",
                },
            ],
        },
    }
}
