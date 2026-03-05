use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState},
    Frame,
};

use crate::state::AppState;

use super::Component;

pub struct ChecklistPanel;

impl Component for ChecklistPanel {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        if !state.checklist.panel_open || state.checklist.is_empty() {
            return;
        }

        let block = Block::default()
            .title("Review Checklist")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state.theme.text_muted));

        let inner = block.inner(area);

        // Split into progress bar and items
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(inner);

        // Render progress bar
        render_progress_bar(frame, layout[0], state);

        // Render checklist items
        render_checklist_items(frame, layout[1], state);

        frame.render_widget(block, area);
    }
}

fn render_progress_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let checked = state.checklist.checked_count();
    let total = state.checklist.total_count();
    let percentage = state.checklist.completion_percentage();

    let progress_text = format!("{}/{} items checked", checked, total);

    let gauge = Gauge::default()
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .gauge_style(Style::default().fg(if percentage == 100.0 {
            Color::Green
        } else if percentage >= 50.0 {
            Color::Yellow
        } else {
            Color::Red
        }))
        .percent(percentage as u16)
        .label(progress_text);

    frame.render_widget(gauge, area);
}

fn render_checklist_items(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
        .checklist
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let checkbox = if item.checked { "[x]" } else { "[ ]" };
            let key_hint = format!("({})", item.key);

            let mut spans = vec![
                Span::styled(
                    checkbox,
                    Style::default().fg(if item.checked {
                        Color::Green
                    } else {
                        Color::Yellow
                    }),
                ),
                Span::raw(" "),
                Span::raw(&item.label),
                Span::raw(" "),
                Span::styled(
                    key_hint,
                    Style::default()
                        .fg(state.theme.text_muted)
                        .add_modifier(Modifier::DIM),
                ),
            ];

            // Add note if present
            if let Some(ref note) = item.note {
                spans.push(Span::raw("\n    "));
                spans.push(Span::styled(
                    format!("Note: {}", note),
                    Style::default()
                        .fg(state.theme.text_muted)
                        .add_modifier(Modifier::ITALIC),
                ));
            }

            let mut line = Line::from(spans);

            // Highlight selected item
            if i == state.checklist.selected {
                line = line.style(
                    Style::default()
                        .bg(state.theme.selection_bg)
                        .fg(state.theme.text),
                );
            }

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Items").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(state.theme.selection_bg)
                .fg(state.theme.text),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(state.checklist.selected));

    frame.render_stateful_widget(list, area, &mut list_state);
}
