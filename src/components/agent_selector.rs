use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::AgentSelectorState;

pub fn render_agent_selector(frame: &mut Frame, selector: &AgentSelectorState) {
    let area = frame.area();
    let dialog_width = 50.min(area.width.saturating_sub(4));
    let dialog_height = (selector.filtered_indices.len() as u16 + 6)
        .min(area.height.saturating_sub(4))
        .max(8);

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    // Note: agent_selector doesn't have access to theme through state,
    // so we keep using Color constants here. The accent color (Cyan) is
    // a reasonable default since the selector is rendered from AgentSelectorState.
    let block = Block::default()
        .title(" Select Agent + Model ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // filter line
            Constraint::Length(1), // separator
            Constraint::Min(1),    // agent list
            Constraint::Length(1), // hints
        ])
        .split(inner);

    // Filter line
    let filter_text = if selector.filter.is_empty() {
        " /filter".to_string()
    } else {
        format!(" /{}\u{2588}", selector.filter)
    };
    let filter_style = if selector.filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    frame.render_widget(Paragraph::new(filter_text).style(filter_style), rows[0]);

    // Separator
    let sep = "\u{2500}".repeat(inner.width as usize);
    frame.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::DarkGray)),
        rows[1],
    );

    // Agent list
    let list_height = rows[2].height as usize;
    let mut lines: Vec<Line> = Vec::new();

    for (vis_idx, &agent_idx) in selector.filtered_indices.iter().enumerate() {
        if vis_idx >= list_height {
            break;
        }
        let agent = &selector.agents[agent_idx];
        let is_selected = vis_idx == selector.selected_agent;

        let prefix = if is_selected { " \u{25b6} " } else { "   " };

        let name_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Show the model selector for the selected agent
        let model_text = if is_selected {
            if agent.models.is_empty() {
                format!("[{}]", agent.default_model)
            } else {
                let model = agent
                    .models
                    .get(selector.selected_model)
                    .unwrap_or(&agent.default_model);
                format!("[{} \u{25b8}]", model)
            }
        } else {
            format!("[{}]", agent.default_model)
        };

        let model_style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, name_style),
            Span::styled(format!("{:<12}", agent.name), name_style),
            Span::styled(model_text, model_style),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "   No agents configured",
            Style::default().fg(Color::DarkGray),
        )));
    }

    frame.render_widget(Paragraph::new(lines), rows[2]);

    // Hints
    let hints = Line::from(vec![
        Span::styled(
            " [j/k]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("agent ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Tab]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("model ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("run ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[3]);
}
