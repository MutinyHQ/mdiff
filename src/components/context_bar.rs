use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::{AppState, DiffViewMode};

use super::Component;

pub struct ContextBar;

impl Component for ContextBar {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;

        let ws_label = if state.diff.options.ignore_whitespace {
            "[ws:off]"
        } else {
            "[ws:on]"
        };

        let view_label = match state.diff.options.view_mode {
            DiffViewMode::Split => "split",
            DiffViewMode::Unified => "unified",
        };

        let line = Line::from(vec![
            Span::styled(" mdiff ", Style::default().fg(Color::Black).bg(theme.accent)),
            Span::raw("  "),
            Span::styled(
                &state.target_label,
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2192} ", Style::default().fg(theme.text_muted)),
            Span::styled(
                "working tree",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("[{view_label}]"),
                Style::default().fg(theme.text_muted),
            ),
            Span::raw(" "),
            Span::styled(ws_label, Style::default().fg(theme.text_muted)),
        ]);

        let bar = Paragraph::new(line).style(Style::default().bg(theme.surface));
        frame.render_widget(bar, area);
    }
}
