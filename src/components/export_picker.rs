use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::export::ExportFormat;
use crate::state::AppState;

pub fn render_export_picker(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.export_format_picker_open {
        return;
    }

    let theme = &state.theme;

    let height = 3;
    let picker_area = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(height + 1),
        width: area.width.saturating_sub(2),
        height,
    };

    frame.render_widget(Clear, picker_area);

    let items: Vec<Span> = ExportFormat::all()
        .iter()
        .flat_map(|fmt| {
            vec![
                Span::styled(
                    format!("[{}]", fmt.shortcut()),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{} ", fmt.label()), Style::default().fg(theme.text)),
            ]
        })
        .collect();

    let title = " Export Format (Esc=cancel) ";
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));
    let paragraph = Paragraph::new(Line::from(items)).block(block);

    frame.render_widget(paragraph, picker_area);
}
