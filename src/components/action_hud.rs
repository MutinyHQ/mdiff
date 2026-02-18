use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::app_state::ActiveView;
use crate::state::AppState;
use crate::theme::Theme;

use super::Component;

pub struct ActionHud;

/// Compute the binding entries for the current state.
fn bindings_for_state(state: &AppState) -> &[(&str, &str)] {
    if state.pty_focus {
        &[("Esc", "exit chat")]
    } else if state.active_view == ActiveView::AgentOutputs {
        &[
            ("j/k", "select"),
            ("Enter", "chat"),
            ("y", "copy"),
            ("^A", "re-run"),
            ("^K", "kill"),
            ("Esc", "back"),
        ]
    } else if state.selection.active {
        &[
            ("j/k", "extend"),
            ("i", "comment"),
            ("d", "delete"),
            ("y", "yank"),
            ("v/Esc", "exit"),
            ("]", "next"),
            ("[", "prev"),
        ]
    } else if state.hud_expanded {
        &[
            ("q", "quit"),
            ("j/k", "nav"),
            ("/", "search"),
            ("Tab", "view"),
            ("w", "ws"),
            ("s", "stage"),
            ("u", "unstage"),
            ("r", "restore"),
            ("c", "commit"),
            ("v", "visual"),
            ("i", "comment"),
            ("a", "annotate"),
            ("y", "yank"),
            ("Space", "expand"),
            ("p", "preview"),
            ("g/G", "top/bot"),
            ("m", "reviewed"),
            ("n", "next unrev"),
            ("R", "refresh"),
            ("t", "target"),
            ("o", "outputs"),
            ("^W", "worktree"),
            ("^A", "agent"),
            (":", "settings"),
            ("?", "hide"),
        ]
    } else {
        &[
            ("q", "quit"),
            ("j/k", "nav"),
            ("/", "search"),
            ("v", "visual"),
            ("i", "comment"),
            ("y", "yank"),
            ("Space", "expand"),
            ("o", "outputs"),
            ("^W", "worktree"),
            ("^A", "agent"),
            (":", "settings"),
            ("?", "help"),
        ]
    }
}

/// Width of a single binding entry: " [key]desc" (leading space + brackets + key + desc).
fn binding_width(key: &str, desc: &str) -> usize {
    // "[key]desc" = 1(bracket) + key.len + 1(bracket) + desc.len
    key.len() + desc.len() + 2
}

/// Build wrapped lines of binding spans that fit within `max_width`.
fn build_lines(
    bindings: &[(&str, &str)],
    max_width: u16,
    ann_text: Option<&str>,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let max_w = max_width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    // Track width with leading " " prefix per line
    let mut current_width: usize = 1; // leading space
    current_spans.push(Span::raw(" ".to_string()));

    for (i, (key, desc)) in bindings.iter().enumerate() {
        let sep_width = if i > 0 { 1 } else { 0 };
        let entry_width = binding_width(key, desc);
        let needed = sep_width + entry_width;

        if current_width + needed > max_w && !current_spans.is_empty() && current_width > 1 {
            // Wrap to new line
            lines.push(Line::from(current_spans));
            current_spans = Vec::new();
            current_spans.push(Span::raw(" ".to_string()));
            current_width = 1;
        } else if i > 0 {
            current_spans.push(Span::styled(
                " ".to_string(),
                Style::default().fg(theme.text_muted),
            ));
            current_width += 1;
        }

        current_spans.push(Span::styled(
            format!("[{key}]"),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
        current_spans.push(Span::styled(
            desc.to_string(),
            Style::default().fg(theme.text_muted),
        ));
        current_width += entry_width;
    }

    // Append annotation count to the last line if it fits
    if let Some(ann) = ann_text {
        let remaining = max_w.saturating_sub(current_width + ann.len());
        if remaining > 0 {
            current_spans.push(Span::raw(" ".repeat(remaining)));
            current_spans.push(Span::styled(
                ann.to_string(),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    lines
}

/// Calculate the number of rows the HUD needs for the given state and width.
pub fn hud_height(state: &AppState, width: u16) -> u16 {
    if state.status_message.is_some() {
        return 1;
    }
    let bindings = bindings_for_state(state);
    let ann_text = annotation_text(state);
    let lines = build_lines(bindings, width, ann_text.as_deref(), &state.theme);
    (lines.len() as u16).max(1)
}

fn annotation_text(state: &AppState) -> Option<String> {
    let count = state.annotations.count();
    if count > 0 {
        Some(format!(" {count} annotations "))
    } else {
        None
    }
}

impl Component for ActionHud {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;

        // Show status message if present, otherwise show keybindings
        if let Some((ref msg, is_error)) = state.status_message {
            let color = if is_error { theme.error } else { theme.success };
            let bar = Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(msg.as_str(), Style::default().fg(color)),
            ]))
            .style(Style::default().bg(theme.surface));
            frame.render_widget(bar, area);
            return;
        }

        let bindings = bindings_for_state(state);
        let ann_text = annotation_text(state);
        let lines = build_lines(bindings, area.width, ann_text.as_deref(), theme);

        let bar = Paragraph::new(lines).style(Style::default().bg(theme.surface));
        frame.render_widget(bar, area);
    }
}
