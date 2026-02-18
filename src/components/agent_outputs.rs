use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::agent_state::{AgentOutputsState, AgentRunStatus};
use crate::state::AppState;
use crate::theme::Theme;

use super::Component;

pub struct AgentOutputs;

impl Component for AgentOutputs {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;

        if state.agent_outputs.runs.is_empty() {
            let block = Block::default()
                .title(" Agent Outputs ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));
            let msg = Paragraph::new(" No agent runs yet. Use [Ctrl+A] to run an agent.")
                .style(Style::default().fg(theme.text_muted))
                .block(block);
            frame.render_widget(msg, area);
            return;
        }

        render_master_detail(frame, area, &state.agent_outputs, theme);
    }
}

fn render_master_detail(frame: &mut Frame, area: Rect, outputs: &AgentOutputsState, theme: &Theme) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_run_list(frame, layout[0], outputs, theme);
    render_run_detail(frame, layout[1], outputs, theme);
}

fn render_run_list(frame: &mut Frame, area: Rect, outputs: &AgentOutputsState, theme: &Theme) {
    let block = Block::default()
        .title(" Runs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let height = inner.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    for (idx, run) in outputs.runs.iter().enumerate() {
        if lines.len() >= height {
            break;
        }

        let is_selected = idx == outputs.selected_run;
        let prefix = if is_selected { "\u{25b6}" } else { " " };

        let (status_icon, status_color) = match &run.status {
            AgentRunStatus::Running => ("\u{25cf}", theme.warning),
            AgentRunStatus::Success { .. } => ("\u{2713}", theme.success),
            AgentRunStatus::Failed { .. } => ("\u{2717}", theme.error),
        };

        let row_style = if is_selected {
            Style::default().bg(theme.selection_bg)
        } else {
            Style::default()
        };

        let status_detail = match &run.status {
            AgentRunStatus::Running => "Running".to_string(),
            AgentRunStatus::Success { exit_code } => format!("Exit {exit_code}"),
            AgentRunStatus::Failed { exit_code } => format!("Exit {exit_code}"),
        };

        // First line: prefix + status + agent/model
        lines.push(Line::from(vec![
            Span::styled(format!("{prefix} "), row_style),
            Span::styled(format!("{status_icon} "), Style::default().fg(status_color)),
            Span::styled(format!("#{} ", run.id), row_style.fg(theme.text_muted)),
            Span::styled(
                format!("{}/{}", run.agent_name, run.model),
                row_style.fg(theme.text),
            ),
        ]));

        // Second line: time + status detail
        if lines.len() < height {
            lines.push(Line::from(vec![
                Span::styled("    ", row_style),
                Span::styled(
                    format!("{} ", &run.started_at[..16.min(run.started_at.len())]),
                    row_style.fg(theme.text_muted),
                ),
                Span::styled(status_detail, row_style.fg(status_color)),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_run_detail(frame: &mut Frame, area: Rect, outputs: &AgentOutputsState, theme: &Theme) {
    let Some(run) = outputs.selected() else {
        let block = Block::default()
            .title(" Output ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.text_muted));
        frame.render_widget(block, area);
        return;
    };

    let title = format!(" Output: #{} {}/{} ", run.id, run.agent_name, run.model);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show the command at the top
    let mut display_lines: Vec<Line> = Vec::new();
    display_lines.push(Line::from(Span::styled(
        format!("$ {}", run.command),
        Style::default()
            .fg(theme.text_muted)
            .add_modifier(Modifier::ITALIC),
    )));
    display_lines.push(Line::from(""));

    // Output lines
    for line in &run.output_lines {
        display_lines.push(Line::from(Span::styled(
            line.clone(),
            Style::default().fg(theme.text),
        )));
    }

    // Show status indicator at end if done
    match &run.status {
        AgentRunStatus::Running => {
            display_lines.push(Line::from(""));
            display_lines.push(Line::from(Span::styled(
                "\u{25cf} Running...",
                Style::default().fg(theme.warning),
            )));
        }
        AgentRunStatus::Success { exit_code } => {
            display_lines.push(Line::from(""));
            display_lines.push(Line::from(Span::styled(
                format!("\u{2713} Process exited with code {exit_code}"),
                Style::default().fg(theme.success),
            )));
        }
        AgentRunStatus::Failed { exit_code } => {
            display_lines.push(Line::from(""));
            display_lines.push(Line::from(Span::styled(
                format!("\u{2717} Process exited with code {exit_code}"),
                Style::default().fg(theme.error),
            )));
        }
    }

    let visible: Vec<Line> = display_lines
        .into_iter()
        .skip(outputs.detail_scroll)
        .take(inner.height as usize)
        .collect();

    let paragraph = Paragraph::new(visible).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
