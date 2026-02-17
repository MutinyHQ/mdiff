use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::{app_state::FocusPanel, AppState};

use super::Component;

pub struct Navigator;

impl Component for Navigator {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focus == FocusPanel::Navigator;

        let border_style = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let visible = state.navigator.visible_entries();
        let total = visible.len();

        let title = if state.navigator.search_active {
            format!(" /{} ", state.navigator.search_query)
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
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(paragraph, area);
            return;
        }

        let inner_height = area.height.saturating_sub(2) as usize;
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
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(40, 40, 50))
                } else if is_active {
                    Style::default().fg(Color::White).bg(Color::Rgb(35, 35, 45))
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected { "\u{25b6}" } else { " " };
                Line::from(vec![
                    Span::styled(format!("{prefix} "), style),
                    Span::styled(entry.display.clone(), style),
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
