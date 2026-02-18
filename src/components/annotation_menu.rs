use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;

pub fn render_annotation_menu(frame: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = frame.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 20.min(area.height.saturating_sub(4)).max(10);

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    // Figure out what line number to show in title from cursor context
    let title = if let Some(item) = state.annotation_menu_items.first() {
        // Use the line from the first item as a representative
        let lineno = item.line_start;
        format!(" Annotations at line {lineno} ")
    } else {
        " Annotations ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    // Calculate how much space we have
    let list_items = state.annotation_menu_items.len() as u16;
    let list_height = list_items.min(inner.height.saturating_sub(4)); // reserve for separator + detail + hints

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(list_height), // annotation list
            Constraint::Length(1),           // separator
            Constraint::Min(1),              // detail area
            Constraint::Length(1),           // hints
        ])
        .split(inner);

    // Annotation list
    let mut lines: Vec<Line> = Vec::new();
    for (idx, item) in state.annotation_menu_items.iter().enumerate() {
        let is_selected = idx == state.annotation_menu_selected;
        let prefix = if is_selected { " \u{25b6} " } else { "   " };

        let range_text = if item.line_start == item.line_end {
            format!("Line {}", item.line_start)
        } else {
            format!("Lines {}-{}", item.line_start, item.line_end)
        };

        // Truncate comment to first line for the list view
        let first_line = item
            .comment
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take((inner.width as usize).saturating_sub(prefix.len() + range_text.len() + 4))
            .collect::<String>();

        let name_style = if is_selected {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        let range_style = if is_selected {
            Style::default().fg(theme.warning)
        } else {
            Style::default().fg(theme.text_muted)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, name_style),
            Span::styled(format!("{range_text}: "), range_style),
            Span::styled(first_line, name_style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), rows[0]);

    // Separator
    let sep = "\u{2500}".repeat(inner.width as usize);
    frame.render_widget(
        Paragraph::new(sep).style(Style::default().fg(theme.text_muted)),
        rows[1],
    );

    // Detail area — full comment of selected annotation
    if let Some(item) = state
        .annotation_menu_items
        .get(state.annotation_menu_selected)
    {
        let detail = Paragraph::new(format!(" {}", item.comment))
            .style(Style::default().fg(theme.text))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, rows[2]);
    }

    // Hints
    let hints = Line::from(vec![
        Span::styled(
            " [j/k]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("navigate ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[e]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("edit ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[d]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("delete ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("close", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[3]);
}
