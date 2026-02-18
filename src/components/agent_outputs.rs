use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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

        render_master_detail(frame, area, state);
    }
}

fn render_master_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_run_list(frame, layout[0], &state.agent_outputs, theme);
    render_run_detail(frame, layout[1], state);
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

fn render_run_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;
    let outputs = &state.agent_outputs;

    let Some(run) = outputs.selected() else {
        let block = Block::default()
            .title(" Output ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.text_muted));
        frame.render_widget(block, area);
        return;
    };

    let title = format!(" Output: #{} {}/{} ", run.id, run.agent_name, run.model);

    // Highlight border when PTY focused
    let border_color = if state.pty_focus {
        theme.warning
    } else {
        theme.accent
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let inner_width = inner.width as usize;
    let inner_height = inner.height as usize;
    if inner_height == 0 || inner_width == 0 {
        return;
    }

    // Render the vt100 terminal screen
    let screen = run.terminal.screen();
    let (term_rows, term_cols) = screen.size();

    // detail_scroll is offset from bottom (0 = live/bottom)
    let scroll_offset = outputs.detail_scroll;

    let mut display_lines: Vec<Line> = Vec::new();

    // Always show the command at top
    display_lines.push(Line::from(Span::styled(
        format!("$ {}", run.command),
        Style::default()
            .fg(theme.text_muted)
            .add_modifier(Modifier::ITALIC),
    )));
    display_lines.push(Line::from(""));

    // Determine how many terminal rows to show
    let lines_for_terminal = inner_height.saturating_sub(display_lines.len());

    // Render visible screen rows (row 0..term_rows)
    // With detail_scroll: 0 = show bottom, higher = scroll up into history
    // For now, render the current visible screen from the vt100 parser
    let rows_to_show = (term_rows as usize).min(lines_for_terminal);
    let first_visible_row = if scroll_offset > 0 {
        // When scrolling up, show earlier rows
        (term_rows as usize).saturating_sub(scroll_offset + rows_to_show)
    } else {
        // At bottom: show the last N rows that fit
        (term_rows as usize).saturating_sub(rows_to_show)
    };

    for row in first_visible_row..first_visible_row + rows_to_show {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_text = String::new();
        let mut current_style = Style::default().fg(theme.text);

        for col in 0..term_cols {
            let cell = screen.cell(row as u16, col);
            if let Some(cell) = cell {
                let cell_style = vt100_cell_to_style(cell, theme);
                let ch = cell.contents();
                let ch = if ch.is_empty() { " " } else { &ch };

                if cell_style == current_style {
                    current_text.push_str(ch);
                } else {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            std::mem::take(&mut current_text),
                            current_style,
                        ));
                    }
                    current_text = ch.to_string();
                    current_style = cell_style;
                }
            }
        }
        if !current_text.is_empty() {
            // Trim trailing spaces
            let trimmed = current_text.trim_end();
            if !trimmed.is_empty() {
                spans.push(Span::styled(trimmed.to_string(), current_style));
            }
        }

        display_lines.push(Line::from(spans));
    }

    // Show status indicator at end if done and we're at the bottom
    if scroll_offset == 0 {
        match &run.status {
            AgentRunStatus::Running => {
                if state.pty_focus {
                    display_lines.push(Line::from(""));
                    display_lines.push(Line::from(Span::styled(
                        "\u{25cf} PTY Focus (Esc to exit)",
                        Style::default().fg(theme.warning),
                    )));
                }
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
    }

    let visible: Vec<Line> = display_lines.into_iter().take(inner_height).collect();
    let paragraph = Paragraph::new(visible).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

/// Convert a vt100 cell to a ratatui Style.
fn vt100_cell_to_style(cell: &vt100::Cell, theme: &Theme) -> Style {
    let mut style = Style::default();

    // Foreground color
    style = style.fg(vt100_color_to_ratatui(cell.fgcolor(), theme.text));

    // Background color
    let bg = vt100_color_to_ratatui(cell.bgcolor(), Color::Reset);
    if bg != Color::Reset {
        style = style.bg(bg);
    }

    // Modifiers
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }

    style
}

/// Convert a vt100::Color to a ratatui::Color.
fn vt100_color_to_ratatui(color: vt100::Color, default: Color) -> Color {
    match color {
        vt100::Color::Default => default,
        vt100::Color::Idx(n) => Color::Indexed(n),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
