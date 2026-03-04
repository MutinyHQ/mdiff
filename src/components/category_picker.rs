use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::annotation_state::{AnnotationCategory, AnnotationSeverity};
use crate::state::app_state::CategoryPickerPhase;
use crate::state::AppState;

pub fn render_category_picker(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.category_picker_open {
        return;
    }

    let theme = &state.theme;

    // Render a centered overlay at the bottom of the screen
    let height = 3;
    let picker_area = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(height + 1),
        width: area.width.saturating_sub(2),
        height,
    };

    frame.render_widget(Clear, picker_area);

    let content = match state.category_picker_phase {
        CategoryPickerPhase::SelectCategory => {
            let items: Vec<Span> = AnnotationCategory::all()
                .iter()
                .flat_map(|cat| {
                    let color = category_color(cat, theme);
                    vec![
                        Span::styled(
                            format!("[{}]", cat.shortcut()),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{} ", cat.label()),
                            Style::default().fg(theme.text),
                        ),
                    ]
                })
                .collect();
            let title = " Category (Enter=Suggestion) ";
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));
            Paragraph::new(Line::from(items)).block(block)
        }
        CategoryPickerPhase::SelectSeverity => {
            let items: Vec<Span> = AnnotationSeverity::all()
                .iter()
                .flat_map(|sev| {
                    let color = severity_color(sev, theme);
                    vec![
                        Span::styled(
                            format!("[{}]", sev.shortcut()),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{} ", sev.label()),
                            Style::default().fg(theme.text),
                        ),
                    ]
                })
                .collect();
            let title = " Severity (Enter=Minor) ";
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));
            Paragraph::new(Line::from(items)).block(block)
        }
    };

    frame.render_widget(content, picker_area);
}

fn category_color(cat: &AnnotationCategory, theme: &crate::theme::Theme) -> Color {
    match cat {
        AnnotationCategory::Bug => theme.error,
        AnnotationCategory::Style => theme.accent,
        AnnotationCategory::Performance => theme.warning,
        AnnotationCategory::Security => Color::Rgb(255, 60, 60),
        AnnotationCategory::Suggestion => theme.success,
        AnnotationCategory::Question => Color::Cyan,
        AnnotationCategory::Nitpick => theme.text_muted,
    }
}

fn severity_color(sev: &AnnotationSeverity, theme: &crate::theme::Theme) -> Color {
    match sev {
        AnnotationSeverity::Critical => theme.error,
        AnnotationSeverity::Major => theme.warning,
        AnnotationSeverity::Minor => theme.text_muted,
        AnnotationSeverity::Info => theme.text_muted,
    }
}
