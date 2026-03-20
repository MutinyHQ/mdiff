use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::state::suggestion_state::SuggestionFocus;
use crate::state::AppState;

use super::text_input::render_text_input;

pub fn render_suggestion_editor(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    let dialog_width = (area.width.saturating_sub(4)).clamp(40, 100);
    let dialog_height = (area.height.saturating_sub(4)).clamp(12, 30);

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let range_label = format_range_label(state);
    let title = format!(" Suggest replacement for {} ", range_label);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ratatui::style::Color::Yellow));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    if state.suggestion.preview_visible {
        render_preview_mode(frame, inner, state);
    } else {
        render_editor_mode(frame, inner, state);
    }
}

fn render_editor_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let comment_focus = state.suggestion.focus == SuggestionFocus::Comment;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // comment bar
            Constraint::Min(4),    // code panes
            Constraint::Length(1), // hints
        ])
        .split(area);

    // Comment bar
    let comment_border_color = if comment_focus {
        ratatui::style::Color::Yellow
    } else {
        theme.text_muted
    };
    let comment_block = Block::default()
        .title(" Comment ")
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(comment_border_color));
    let comment_inner = comment_block.inner(rows[0]);
    frame.render_widget(comment_block, rows[0]);

    if comment_focus {
        render_text_input(
            frame,
            comment_inner,
            state.suggestion.comment.text(),
            state.suggestion.comment.cursor_char_index(),
            Style::default().fg(theme.text),
        );
    } else {
        let comment_text = if state.suggestion.comment.is_empty() {
            "(optional comment — Shift+Tab to focus)"
        } else {
            state.suggestion.comment.text()
        };
        let style = if state.suggestion.comment.is_empty() {
            Style::default().fg(theme.text_muted)
        } else {
            Style::default().fg(theme.text)
        };
        frame.render_widget(Paragraph::new(comment_text).style(style), comment_inner);
    }

    // Determine if we can fit side-by-side or need to stack
    let min_pane_width = 20;
    let side_by_side = area.width >= (min_pane_width * 2 + 3);

    if side_by_side {
        render_side_by_side_panes(frame, rows[1], state);
    } else {
        render_stacked_panes(frame, rows[1], state);
    }

    // Hints bar
    let replacement_focus = state.suggestion.focus == SuggestionFocus::Replacement;
    let hints = Line::from(vec![
        Span::styled(
            " Ctrl+Enter",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Save  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "Tab",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Preview  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "S-Tab",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if replacement_focus {
                ":Comment  "
            } else {
                ":Editor  "
            },
            Style::default().fg(theme.text_muted),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Cancel", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[2]);
}

fn render_side_by_side_panes(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let replacement_focus = state.suggestion.focus == SuggestionFocus::Replacement;

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left pane: original code (read-only)
    let left_block = Block::default()
        .title(" Original (read-only) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.text_muted));
    let left_inner = left_block.inner(cols[0]);
    frame.render_widget(left_block, cols[0]);

    let original_lines: Vec<Line> = state
        .suggestion
        .original_code
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                l.as_str(),
                Style::default().fg(theme.text_muted),
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(original_lines).wrap(Wrap { trim: false }),
        left_inner,
    );

    // Right pane: replacement editor
    let right_border_color = if replacement_focus {
        ratatui::style::Color::Yellow
    } else {
        theme.text_muted
    };
    let right_block = Block::default()
        .title(" Replacement (edit) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(right_border_color));
    let right_inner = right_block.inner(cols[1]);
    frame.render_widget(right_block, cols[1]);

    if replacement_focus {
        render_text_input(
            frame,
            right_inner,
            state.suggestion.replacement.text(),
            state.suggestion.replacement.cursor_char_index(),
            Style::default().fg(theme.text),
        );
    } else {
        let text = state.suggestion.replacement.text();
        frame.render_widget(
            Paragraph::new(text)
                .style(Style::default().fg(theme.text))
                .wrap(Wrap { trim: false }),
            right_inner,
        );
    }
}

fn render_stacked_panes(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let replacement_focus = state.suggestion.focus == SuggestionFocus::Replacement;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Top: original
    let top_block = Block::default()
        .title(" Original (read-only) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.text_muted));
    let top_inner = top_block.inner(rows[0]);
    frame.render_widget(top_block, rows[0]);

    let original_lines: Vec<Line> = state
        .suggestion
        .original_code
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                l.as_str(),
                Style::default().fg(theme.text_muted),
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(original_lines).wrap(Wrap { trim: false }),
        top_inner,
    );

    // Bottom: replacement editor
    let bottom_border_color = if replacement_focus {
        ratatui::style::Color::Yellow
    } else {
        theme.text_muted
    };
    let bottom_block = Block::default()
        .title(" Replacement (edit) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(bottom_border_color));
    let bottom_inner = bottom_block.inner(rows[1]);
    frame.render_widget(bottom_block, rows[1]);

    if replacement_focus {
        render_text_input(
            frame,
            bottom_inner,
            state.suggestion.replacement.text(),
            state.suggestion.replacement.cursor_char_index(),
            Style::default().fg(theme.text),
        );
    } else {
        let text = state.suggestion.replacement.text();
        frame.render_widget(
            Paragraph::new(text)
                .style(Style::default().fg(theme.text))
                .wrap(Wrap { trim: false }),
            bottom_inner,
        );
    }
}

fn render_preview_mode(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let preview_block = Block::default()
        .title(" Preview (unified diff) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ratatui::style::Color::Yellow));
    let preview_inner = preview_block.inner(rows[0]);
    frame.render_widget(preview_block, rows[0]);

    let mut diff_lines: Vec<Line> = Vec::new();

    for line in &state.suggestion.original_code {
        diff_lines.push(Line::from(Span::styled(
            format!("- {}", line),
            Style::default().fg(ratatui::style::Color::Red),
        )));
    }

    let replacement_text = state.suggestion.replacement.text();
    for line in replacement_text.lines() {
        diff_lines.push(Line::from(Span::styled(
            format!("+ {}", line),
            Style::default().fg(ratatui::style::Color::Green),
        )));
    }
    // Handle empty replacement (delete lines)
    if replacement_text.is_empty() && !state.suggestion.original_code.is_empty() {
        diff_lines.push(Line::from(Span::styled(
            "(lines will be deleted)",
            Style::default()
                .fg(theme.text_muted)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    frame.render_widget(
        Paragraph::new(diff_lines).wrap(Wrap { trim: false }),
        preview_inner,
    );

    // Hints
    let hints = Line::from(vec![
        Span::styled(
            " Tab",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Back to editor  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "Ctrl+Enter",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Save  ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(":Cancel", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[1]);
}

fn format_range_label(state: &AppState) -> String {
    match (state.suggestion.old_range, state.suggestion.new_range) {
        (_, Some((s, e))) if s == e => format!("line {}", s),
        (_, Some((s, e))) => format!("lines {}-{}", s, e),
        (Some((s, e)), None) if s == e => format!("removed line {} (old)", s),
        (Some((s, e)), None) => format!("removed lines {}-{} (old)", s, e),
        (None, None) => "selected lines".to_string(),
    }
}
