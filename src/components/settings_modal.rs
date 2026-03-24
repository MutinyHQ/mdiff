use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::config::MdiffConfig;
use crate::state::settings_state::SETTINGS_ROW_COUNT;
use crate::state::AppState;
use crate::state::DiffViewMode;

pub fn render_settings_modal(frame: &mut Frame, state: &AppState, config: &MdiffConfig) {
    let area = frame.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = (SETTINGS_ROW_COUNT as u16 + 4).min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let theme = &state.theme;

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let constraints: Vec<Constraint> = (0..SETTINGS_ROW_COUNT)
        .map(|_| Constraint::Length(1))
        .chain(std::iter::once(Constraint::Length(1))) // hints row
        .chain(std::iter::once(Constraint::Min(0))) // spacer
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let selected = state.settings.selected_row;

    // Row 0: Theme
    let theme_value = format!("< {} >", state.theme.name);
    render_setting_row(frame, rows[0], "Theme", &theme_value, selected == 0, theme);

    // Row 1: View Mode
    let view_value = match state.diff.options.view_mode {
        DiffViewMode::Split => "< Split >",
        DiffViewMode::Unified => "< Unified >",
    };
    render_setting_row(
        frame,
        rows[1],
        "View Mode",
        view_value,
        selected == 1,
        theme,
    );

    // Row 2: Ignore Whitespace
    let ws_value = if state.diff.options.ignore_whitespace {
        "[x]"
    } else {
        "[ ]"
    };
    render_setting_row(
        frame,
        rows[2],
        "Ignore Whitespace",
        ws_value,
        selected == 2,
        theme,
    );

    // Row 3: Context Lines
    let ctx_value = format!("< {} >", state.diff.display_context);
    render_setting_row(
        frame,
        rows[3],
        "Context Lines",
        &ctx_value,
        selected == 3,
        theme,
    );

    // Row 4: File View Mode (Tree/Flat)
    let tree_value = if state.navigator.tree_mode {
        "< Tree >"
    } else {
        "< Flat >"
    };
    render_setting_row(
        frame,
        rows[4],
        "File View",
        tree_value,
        selected == 4,
        theme,
    );

    // Row 5: Parent Agent Provider
    let parent_provider = config.agentic_review.resolved_parent_provider();
    let pp_value = format!("< {} >", parent_provider);
    render_setting_row(
        frame,
        rows[5],
        "Review Provider",
        &pp_value,
        selected == 5,
        theme,
    );

    // Row 6: Parent Agent Model
    let parent_value = format!("< {} >", config.agentic_review.parent_model);
    render_setting_row(
        frame,
        rows[6],
        "Review Model",
        &parent_value,
        selected == 6,
        theme,
    );

    // Row 7: Sub-agent Provider
    let child_provider = config.agentic_review.resolved_child_provider();
    let cp_value = format!("< {} >", child_provider);
    render_setting_row(
        frame,
        rows[7],
        "Sub-agent Provider",
        &cp_value,
        selected == 7,
        theme,
    );

    // Row 8: Sub-agent Model
    let child_value = format!("< {} >", config.agentic_review.child_model);
    render_setting_row(
        frame,
        rows[8],
        "Sub-agent Model",
        &child_value,
        selected == 8,
        theme,
    );

    // Row 9: Max Agent Turns
    let turns_value = format!("< {} >", config.agentic_review.max_agent_turns);
    render_setting_row(
        frame,
        rows[9],
        "Max Agent Turns",
        &turns_value,
        selected == 9,
        theme,
    );

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
            "[h/l]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("change ", Style::default().fg(theme.text_muted)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("close", Style::default().fg(theme.text_muted)),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[SETTINGS_ROW_COUNT]);
}

fn render_setting_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_selected: bool,
    theme: &crate::theme::Theme,
) {
    let label_style = if is_selected {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let value_style = if is_selected {
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let prefix = if is_selected { " \u{25b6} " } else { "   " };

    // Pad label to align values
    let padded_label = format!("{:<20}", label);

    let line = Line::from(vec![
        Span::styled(prefix, label_style),
        Span::styled(padded_label, label_style),
        Span::styled(value.to_string(), value_style),
    ]);

    let bg = if is_selected {
        Style::default().bg(theme.selection_bg)
    } else {
        Style::default().bg(Color::Reset)
    };

    frame.render_widget(Paragraph::new(line).style(bg), area);
}
