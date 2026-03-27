use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::display_map::DisplayRowInfo;
use crate::git::types::DiffLineOrigin;
use crate::state::AppState;
use crate::theme::Theme;

/// Classification of a minimap row based on the change types it covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MinimapCell {
    Empty,
    Addition,
    Deletion,
    Mixed,
}

/// Unicode block characters for intensity encoding.
const LIGHT_SHADE: &str = "\u{2591}";
const MEDIUM_SHADE: &str = "\u{2592}";
const FULL_BLOCK: &str = "\u{2588}";

fn intensity_char(change_ratio: f64) -> &'static str {
    if change_ratio <= 0.0 {
        " "
    } else if change_ratio < 0.33 {
        LIGHT_SHADE
    } else if change_ratio < 0.66 {
        MEDIUM_SHADE
    } else {
        FULL_BLOCK
    }
}

fn cell_color(cell: MinimapCell, theme: &Theme) -> Color {
    match cell {
        MinimapCell::Empty => theme.text_muted,
        MinimapCell::Addition => theme.diff_add_fg,
        MinimapCell::Deletion => theme.diff_del_fg,
        MinimapCell::Mixed => Color::Yellow,
    }
}

/// Compute minimap data from the display map.
#[derive(Clone)]
struct MinimapRow {
    cell: MinimapCell,
    change_ratio: f64,
}

fn compute_minimap_rows(display_map: &[DisplayRowInfo], minimap_height: usize) -> Vec<MinimapRow> {
    if display_map.is_empty() || minimap_height == 0 {
        return vec![
            MinimapRow {
                cell: MinimapCell::Empty,
                change_ratio: 0.0,
            };
            minimap_height
        ];
    }

    let total_lines = display_map.len();
    let lines_per_row = (total_lines as f64 / minimap_height as f64).ceil() as usize;
    let lines_per_row = lines_per_row.max(1);

    let mut rows = Vec::with_capacity(minimap_height);

    for row_idx in 0..minimap_height {
        let start = row_idx * lines_per_row;
        let end = ((row_idx + 1) * lines_per_row).min(total_lines);

        if start >= total_lines {
            rows.push(MinimapRow {
                cell: MinimapCell::Empty,
                change_ratio: 0.0,
            });
            continue;
        }

        let mut additions = 0usize;
        let mut deletions = 0usize;
        let mut total = 0usize;

        for info in &display_map[start..end] {
            total += 1;
            match info.origin {
                Some(DiffLineOrigin::Addition) => additions += 1,
                Some(DiffLineOrigin::Deletion) => deletions += 1,
                _ => {}
            }
        }

        let changes = additions + deletions;
        let change_ratio = if total > 0 {
            changes as f64 / total as f64
        } else {
            0.0
        };

        let cell = if additions > 0 && deletions > 0 {
            MinimapCell::Mixed
        } else if additions > 0 {
            MinimapCell::Addition
        } else if deletions > 0 {
            MinimapCell::Deletion
        } else {
            MinimapCell::Empty
        };

        rows.push(MinimapRow { cell, change_ratio });
    }

    rows
}

/// Render the minimap widget into the given area.
pub fn render_minimap(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    display_map: &[DisplayRowInfo],
) {
    let theme = &state.theme;
    let minimap_height = area.height as usize;

    if minimap_height == 0 {
        return;
    }

    let rows = compute_minimap_rows(display_map, minimap_height);
    let total_display_rows = display_map.len();

    let viewport_height = area.height as usize;
    let scroll_offset = state.diff.scroll_offset;

    let lines_per_row = if total_display_rows > 0 && minimap_height > 0 {
        (total_display_rows as f64 / minimap_height as f64).ceil() as usize
    } else {
        1
    };

    let viewport_start_row = if lines_per_row > 0 {
        scroll_offset / lines_per_row
    } else {
        0
    };
    let viewport_end_row = if lines_per_row > 0 {
        ((scroll_offset + viewport_height) / lines_per_row).min(minimap_height)
    } else {
        minimap_height
    };

    let width = area.width as usize;
    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let in_viewport = idx >= viewport_start_row && idx < viewport_end_row;
            let ch = intensity_char(row.change_ratio);
            let fg = cell_color(row.cell, theme);

            let style = if in_viewport {
                Style::default().fg(fg).bg(theme.accent)
            } else {
                Style::default().fg(fg).bg(theme.surface)
            };

            let content: String = ch.repeat(width);
            Line::from(Span::styled(content, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display_map::DisplayRowInfo;
    use crate::git::types::DiffLineOrigin;

    fn make_row_info(origin: Option<DiffLineOrigin>) -> DisplayRowInfo {
        DisplayRowInfo {
            hunk_index: 0,
            line_index: None,
            old_lineno: None,
            new_lineno: None,
            origin,
            is_header: false,
            is_collapsed_indicator: false,
            gap_id: None,
            hidden_count: 0,
            expand_direction: None,
        }
    }

    #[test]
    fn empty_display_map_produces_empty_rows() {
        let rows = compute_minimap_rows(&[], 5);
        assert_eq!(rows.len(), 5);
        for row in &rows {
            assert_eq!(row.cell, MinimapCell::Empty);
            assert_eq!(row.change_ratio, 0.0);
        }
    }

    #[test]
    fn all_additions_produce_addition_cells() {
        let display_map: Vec<DisplayRowInfo> = (0..10)
            .map(|_| make_row_info(Some(DiffLineOrigin::Addition)))
            .collect();
        let rows = compute_minimap_rows(&display_map, 5);
        assert_eq!(rows.len(), 5);
        for row in &rows {
            assert_eq!(row.cell, MinimapCell::Addition);
            assert!(row.change_ratio > 0.0);
        }
    }

    #[test]
    fn mixed_changes_produce_mixed_cells() {
        let display_map: Vec<DisplayRowInfo> = vec![
            make_row_info(Some(DiffLineOrigin::Addition)),
            make_row_info(Some(DiffLineOrigin::Deletion)),
        ];
        let rows = compute_minimap_rows(&display_map, 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cell, MinimapCell::Mixed);
    }

    #[test]
    fn context_only_produces_empty_cells() {
        let display_map: Vec<DisplayRowInfo> = (0..10)
            .map(|_| make_row_info(Some(DiffLineOrigin::Context)))
            .collect();
        let rows = compute_minimap_rows(&display_map, 5);
        for row in &rows {
            assert_eq!(row.cell, MinimapCell::Empty);
            assert_eq!(row.change_ratio, 0.0);
        }
    }

    #[test]
    fn minimap_height_larger_than_display_map() {
        let display_map: Vec<DisplayRowInfo> = (0..3)
            .map(|_| make_row_info(Some(DiffLineOrigin::Addition)))
            .collect();
        let rows = compute_minimap_rows(&display_map, 10);
        assert_eq!(rows.len(), 10);
        let non_empty = rows.iter().filter(|r| r.cell != MinimapCell::Empty).count();
        assert!(non_empty > 0);
    }
}
