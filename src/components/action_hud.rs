use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::app_state::{ActiveView, FocusPanel};
use crate::state::AppState;
use crate::theme::Theme;

use super::Component;

pub struct ActionHud;

/// Compute the binding entries for the current state.
fn bindings_for_state(state: &AppState) -> &[(&str, &str)] {
    if state.pty_focus {
        &[("^W", "window")]
    } else if state.active_view == ActiveView::AgentOutputs {
        match state.focus {
            FocusPanel::AgentOutput => &[("j/k", "scroll"), ("^W h", "list"), ("o", "back")],
            _ => &[
                ("j/k", "select"),
                ("y", "copy"),
                ("w", "worktree"),
                ("^K", "kill"),
                ("^W l", "output"),
                ("o", "back"),
            ],
        }
    } else if state.active_view == ActiveView::WorktreeBrowser {
        &[
            ("j/k", "nav"),
            ("Enter", "open"),
            ("r", "refresh"),
            ("f", "freeze"),
            ("Esc", "back"),
        ]
    } else if state.active_view == ActiveView::FeedbackSummary {
        &[
            ("j/k", "scroll"),
            ("y", "json"),
            ("p", "prompt"),
            ("Esc/F", "back"),
        ]
    } else if state.selection.active {
        &[
            ("j/k", "extend"),
            ("i", "comment"),
            ("d", "delete"),
            ("y", "yank"),
            ("v/Esc", "exit"),
            ("^]/^[", "next/prev"),
        ]
    } else if state.focus == FocusPanel::Navigator {
        if state.hud_expanded {
            &[
                ("^C/D", "quit"),
                ("j/k", "nav"),
                ("/", "search"),
                ("Enter", "diff"),
                ("g/G", "top/bot"),
                ("m", "reviewed"),
                ("n", "next unrev"),
                ("s", "stage"),
                ("u", "unstage"),
                ("r", "restore"),
                ("c", "commit"),
                ("t", "target"),
                ("T", "tree"),
                ("C", "checklist"),
                ("F", "summary"),
                ("R", "refresh"),
                ("o", "outputs"),
                ("^W", "window"),
                ("^A", "agent"),
                (":", "command"),
                ("?", "hide"),
            ]
        } else {
            &[
                ("^C/D", "quit"),
                ("j/k", "nav"),
                ("/", "search"),
                ("Enter", "diff"),
                ("o", "outputs"),
                ("^W", "window"),
                ("^A", "agent"),
                (":", "command"),
                ("?", "help"),
            ]
        }
    } else if state.focus == FocusPanel::DiffView {
        if state.hud_expanded {
            &[
                ("^C/D", "quit"),
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
                ("b", "bookmark"),
                ("B", "bookmarks"),
                ("e", "editor"),
                ("t", "target"),
                ("C", "checklist"),
                ("F", "summary"),
                ("R", "refresh"),
                ("o", "outputs"),
                ("^W", "window"),
                ("^A", "agent"),
                (":", "command"),
                ("?", "hide"),
            ]
        } else {
            &[
                ("^C/D", "quit"),
                ("j/k", "nav"),
                ("/", "search"),
                ("v", "visual"),
                ("i", "comment"),
                ("y", "yank"),
                ("Space", "expand"),
                ("o", "outputs"),
                ("^W", "window"),
                ("^A", "agent"),
                (":", "command"),
                ("?", "help"),
            ]
        }
    } else if state.hud_expanded {
        &[
            ("^C/D", "quit"),
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
            ("^W", "window"),
            ("^A", "agent"),
            (":", "command"),
            ("?", "hide"),
        ]
    } else {
        &[
            ("^C/D", "quit"),
            ("j/k", "nav"),
            ("/", "search"),
            ("v", "visual"),
            ("i", "comment"),
            ("y", "yank"),
            ("Space", "expand"),
            ("o", "outputs"),
            ("^W", "window"),
            ("^A", "agent"),
            (":", "command"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::event::{map_key_to_action, KeyContext};
    use crate::state::app_state::FocusPanel;
    use crate::state::{AppState, DiffOptions};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_state() -> AppState {
        AppState::new(DiffOptions::new(false, false), Theme::from_name("one-dark"))
    }

    fn key_context(state: &AppState) -> KeyContext {
        KeyContext {
            focus: state.focus,
            search_active: state.navigator.search_active,
            diff_search_active: state.diff.search_active,
            global_search_active: state.global_search.active,
            commit_dialog_open: state.commit_dialog_open,
            target_dialog_open: state.target_dialog_open,
            comment_editor_open: state.comment_editor_open,
            category_picker_open: state.category_picker_open,
            category_picker_phase: state.category_picker_phase,
            agent_selector_open: state.agent_selector.open,
            annotation_menu_open: state.annotation_menu_open,
            restore_confirm_open: state.restore_confirm_open,
            settings_open: state.settings.open,
            visual_mode_active: state.selection.active,
            active_view: state.active_view,
            pty_focus: state.pty_focus,
            checklist_panel_open: state.checklist.panel_open,
            bookmark_list_open: state.bookmarks.list_visible,
            which_key_visible: state.which_key_visible,
            tree_mode: state.navigator.tree_mode,
            tree_z_pending: false,
            bracket_pending: None,
            mark_pending: false,
            jump_mark_pending: false,
            command_bar_active: state.command_bar.active,
            file_picker_active: state.file_picker.active,
            agentic_review_modal_open: state.agentic_review_modal_open,
            agentic_review_panel_open: state.agentic_review_panel_open,
            agentic_review_composing: state.agentic_review_composing,
            window_pending: false,
            qa_input_open: state.qa.input_open,
            qa_answer_visible: state.qa.answer_visible,
        }
    }

    fn action_for_key(state: &AppState, key: KeyEvent) -> Option<Action> {
        map_key_to_action(key, &key_context(state))
    }

    fn parse_key(binding: &str) -> Option<(KeyModifiers, KeyCode)> {
        match binding {
            "Tab" => Some((KeyModifiers::NONE, KeyCode::Tab)),
            "Enter" => Some((KeyModifiers::NONE, KeyCode::Enter)),
            "Esc" => Some((KeyModifiers::NONE, KeyCode::Esc)),
            "Space" => Some((KeyModifiers::NONE, KeyCode::Char(' '))),
            _ => {
                if let Some(rest) = binding.strip_prefix('^') {
                    Some((
                        KeyModifiers::CONTROL,
                        KeyCode::Char(rest.chars().next()?.to_ascii_lowercase()),
                    ))
                } else {
                    Some((KeyModifiers::NONE, KeyCode::Char(binding.chars().next()?)))
                }
            }
        }
    }

    fn binding_exists(state: &AppState, binding: &str) -> bool {
        if let Some(rest) = binding.strip_prefix("^W ") {
            let ctrl_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
            if !matches!(action_for_key(state, ctrl_w), Some(Action::WindowPrefix)) {
                return false;
            }
            let mut ctx = key_context(state);
            ctx.window_pending = true;
            let key = KeyEvent::new(
                KeyCode::Char(rest.chars().next().unwrap()),
                KeyModifiers::NONE,
            );
            return map_key_to_action(key, &ctx).is_some();
        }

        if binding.len() > 1 && binding.contains('/') {
            return binding.split('/').any(|part| binding_exists(state, part));
        }

        let Some((modifiers, key)) = parse_key(binding) else {
            return false;
        };

        action_for_key(state, KeyEvent::new(key, modifiers)).is_some()
    }

    fn assert_hud_bindings_exist(state: &AppState) {
        for (binding, _) in bindings_for_state(state) {
            assert!(
                binding_exists(state, binding),
                "HUD advertised missing binding: {binding}"
            );
        }
    }

    #[test]
    fn navigator_compact_hud_bindings_exist() {
        let mut state = make_state();
        state.focus = FocusPanel::Navigator;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn navigator_expanded_hud_bindings_exist() {
        let mut state = make_state();
        state.focus = FocusPanel::Navigator;
        state.hud_expanded = true;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn diff_view_compact_hud_bindings_exist() {
        let mut state = make_state();
        state.focus = FocusPanel::DiffView;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn diff_view_expanded_hud_bindings_exist() {
        let mut state = make_state();
        state.focus = FocusPanel::DiffView;
        state.hud_expanded = true;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn selection_hud_bindings_exist() {
        let mut state = make_state();
        state.focus = FocusPanel::DiffView;
        state.selection.active = true;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn agent_output_run_list_hud_bindings_exist() {
        let mut state = make_state();
        state.active_view = ActiveView::AgentOutputs;
        state.focus = FocusPanel::AgentRunList;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn agent_output_detail_hud_bindings_exist() {
        let mut state = make_state();
        state.active_view = ActiveView::AgentOutputs;
        state.focus = FocusPanel::AgentOutput;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn agent_output_pty_hud_bindings_exist() {
        let mut state = make_state();
        state.active_view = ActiveView::AgentOutputs;
        state.focus = FocusPanel::AgentOutput;
        state.pty_focus = true;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn worktree_browser_hud_bindings_exist() {
        let mut state = make_state();
        state.active_view = ActiveView::WorktreeBrowser;
        assert_hud_bindings_exist(&state);
    }

    #[test]
    fn feedback_summary_hud_bindings_exist() {
        let mut state = make_state();
        state.active_view = ActiveView::FeedbackSummary;
        assert_hud_bindings_exist(&state);
    }
}
