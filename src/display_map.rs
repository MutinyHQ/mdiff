use std::collections::HashMap;

use crate::git::types::{DiffLine, DiffLineOrigin, FileDelta};
use crate::state::DiffViewMode;

/// Direction a collapsed indicator expands toward when activated.
#[derive(Debug, Clone, Copy)]
pub enum ExpandDirection {
    Down, // ▼ — reveals lines below (after previous change)
    Up,   // ▲ — reveals lines above (before next change)
}

/// A filtered item from a hunk: either a visible line or a collapsed indicator.
pub enum FilteredItem<'a> {
    Line {
        line: &'a DiffLine,
        hunk_line_index: usize,
    },
    CollapsedIndicator {
        hidden_count: usize,
        gap_id: usize,
        direction: ExpandDirection,
    },
}

/// Filter a hunk's lines, collapsing context runs that exceed the display window.
///
/// Returns `(filtered_items, next_gap_id_offset)`.
pub fn filter_hunk_lines<'a>(
    lines: &'a [DiffLine],
    display_context: usize,
    gap_expansions: &HashMap<usize, usize>,
    gap_id_offset: usize,
) -> (Vec<FilteredItem<'a>>, usize) {
    let mut items = Vec::new();
    let mut gap_id = gap_id_offset;
    let mut i = 0;

    while i < lines.len() {
        if lines[i].origin != DiffLineOrigin::Context {
            items.push(FilteredItem::Line {
                line: &lines[i],
                hunk_line_index: i,
            });
            i += 1;
            continue;
        }

        // Collect the full run of consecutive context lines
        let run_start = i;
        while i < lines.len() && lines[i].origin == DiffLineOrigin::Context {
            i += 1;
        }
        let run_end = i;
        let total = run_end - run_start;

        // Determine neighbors
        let has_change_before = run_start > 0;
        let has_change_after = run_end < lines.len();

        // Compute baseline visible lines (without expansion)
        let show_after_prev = if has_change_before {
            display_context
        } else {
            0
        };
        let show_before_next = if has_change_after { display_context } else { 0 };
        let baseline_total = show_after_prev + show_before_next;

        if baseline_total >= total {
            // Show all context lines, no gap possible
            for (idx, line) in lines[run_start..run_end].iter().enumerate() {
                items.push(FilteredItem::Line {
                    line,
                    hunk_line_index: run_start + idx,
                });
            }
        } else if has_change_before && has_change_after {
            // Between two diffs: two indicators with independent expansion.
            // Top indicator expands downward, bottom indicator expands upward.
            let top_gap_id = gap_id;
            let bottom_gap_id = gap_id + 1;
            let top_extra = gap_expansions.get(&top_gap_id).copied().unwrap_or(0);
            let bottom_extra = gap_expansions.get(&bottom_gap_id).copied().unwrap_or(0);
            let show_top = show_after_prev + top_extra;
            let show_bottom = show_before_next + bottom_extra;
            let total_show = show_top + show_bottom;

            if total_show >= total {
                for (idx, line) in lines[run_start..run_end].iter().enumerate() {
                    items.push(FilteredItem::Line {
                        line,
                        hunk_line_index: run_start + idx,
                    });
                }
            } else {
                let hidden = total - total_show;

                // Lines after previous change (top side)
                let first_end = run_start + show_top;
                for (idx, line) in lines[run_start..first_end].iter().enumerate() {
                    items.push(FilteredItem::Line {
                        line,
                        hunk_line_index: run_start + idx,
                    });
                }

                // Top indicator (expands downward)
                items.push(FilteredItem::CollapsedIndicator {
                    hidden_count: hidden,
                    gap_id: top_gap_id,
                    direction: ExpandDirection::Down,
                });

                // Bottom indicator (expands upward)
                items.push(FilteredItem::CollapsedIndicator {
                    hidden_count: hidden,
                    gap_id: bottom_gap_id,
                    direction: ExpandDirection::Up,
                });

                // Lines before next change (bottom side)
                let last_start = run_end - show_bottom;
                for (idx, line) in lines[last_start..run_end].iter().enumerate() {
                    items.push(FilteredItem::Line {
                        line,
                        hunk_line_index: last_start + idx,
                    });
                }
            }

            gap_id += 2;
        } else {
            // Single-sided gap: above a diff (!has_change_before) or below a diff
            // (!has_change_after). One indicator.
            let extra = gap_expansions.get(&gap_id).copied().unwrap_or(0);
            let (show_top, show_bottom) = if !has_change_before {
                // Above a diff: expand from bottom upward
                (show_after_prev, show_before_next + extra)
            } else {
                // Below a diff: expand from top downward
                (show_after_prev + extra, show_before_next)
            };
            let total_show = show_top + show_bottom;

            if total_show >= total {
                for (idx, line) in lines[run_start..run_end].iter().enumerate() {
                    items.push(FilteredItem::Line {
                        line,
                        hunk_line_index: run_start + idx,
                    });
                }
            } else {
                let first_end = run_start + show_top;
                for (idx, line) in lines[run_start..first_end].iter().enumerate() {
                    items.push(FilteredItem::Line {
                        line,
                        hunk_line_index: run_start + idx,
                    });
                }

                let hidden = total - total_show;
                let direction = if !has_change_before {
                    ExpandDirection::Up
                } else {
                    ExpandDirection::Down
                };
                items.push(FilteredItem::CollapsedIndicator {
                    hidden_count: hidden,
                    gap_id,
                    direction,
                });

                let last_start = run_end - show_bottom;
                for (idx, line) in lines[last_start..run_end].iter().enumerate() {
                    items.push(FilteredItem::Line {
                        line,
                        hunk_line_index: last_start + idx,
                    });
                }
            }

            gap_id += 1;
        }
    }

    (items, gap_id)
}

/// Information about what a single display row maps to in the diff.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DisplayRowInfo {
    /// Index of the hunk this row belongs to.
    pub hunk_index: usize,
    /// Index of the line within the hunk (None for hunk headers and collapsed indicators).
    pub line_index: Option<usize>,
    /// Old-side line number (1-based), if applicable.
    pub old_lineno: Option<u32>,
    /// New-side line number (1-based), if applicable.
    pub new_lineno: Option<u32>,
    /// The origin type of this line.
    pub origin: Option<DiffLineOrigin>,
    /// Whether this is a hunk header row.
    pub is_header: bool,
    /// Whether this row is a collapsed context indicator.
    pub is_collapsed_indicator: bool,
    /// The gap_id for collapsed indicators (used for expansion).
    pub gap_id: Option<usize>,
    /// Number of hidden lines (only meaningful for collapsed indicators).
    pub hidden_count: usize,
    /// Expand direction for collapsed indicators.
    pub expand_direction: Option<ExpandDirection>,
}

/// Build a display map for the split view.
pub fn build_split_display_map(
    delta: &FileDelta,
    display_context: usize,
    gap_expansions: &HashMap<usize, usize>,
) -> Vec<DisplayRowInfo> {
    let mut rows = Vec::new();
    let mut gap_id_offset = 0;

    for (hunk_idx, hunk) in delta.hunks.iter().enumerate() {
        // Hunk header row
        rows.push(DisplayRowInfo {
            hunk_index: hunk_idx,
            line_index: None,
            old_lineno: None,
            new_lineno: None,
            origin: None,
            is_header: true,
            is_collapsed_indicator: false,
            gap_id: None,
            hidden_count: 0,
        });

        let (items, next_offset) =
            filter_hunk_lines(&hunk.lines, display_context, gap_expansions, gap_id_offset);
        gap_id_offset = next_offset;

        let mut i = 0;
        while i < items.len() {
            match &items[i] {
                FilteredItem::CollapsedIndicator {
                    hidden_count,
                    gap_id,
                } => {
                    rows.push(DisplayRowInfo {
                        hunk_index: hunk_idx,
                        line_index: None,
                        old_lineno: None,
                        new_lineno: None,
                        origin: None,
                        is_header: false,
                        is_collapsed_indicator: true,
                        gap_id: Some(*gap_id),
                        hidden_count: *hidden_count,
                    });
                    i += 1;
                }
                FilteredItem::Line {
                    line,
                    hunk_line_index,
                } => match line.origin {
                    DiffLineOrigin::Context => {
                        rows.push(DisplayRowInfo {
                            hunk_index: hunk_idx,
                            line_index: Some(*hunk_line_index),
                            old_lineno: line.old_lineno,
                            new_lineno: line.new_lineno,
                            origin: Some(DiffLineOrigin::Context),
                            is_header: false,
                            is_collapsed_indicator: false,
                            gap_id: None,
                            hidden_count: 0,
                        });
                        i += 1;
                    }
                    DiffLineOrigin::Deletion => {
                        // Collect consecutive deletions
                        let del_start = i;
                        while i < items.len() {
                            if let FilteredItem::Line { line: l, .. } = &items[i] {
                                if l.origin == DiffLineOrigin::Deletion {
                                    i += 1;
                                    continue;
                                }
                            }
                            break;
                        }
                        // Collect consecutive additions
                        let add_start = i;
                        while i < items.len() {
                            if let FilteredItem::Line { line: l, .. } = &items[i] {
                                if l.origin == DiffLineOrigin::Addition {
                                    i += 1;
                                    continue;
                                }
                            }
                            break;
                        }

                        let dels: Vec<_> = items[del_start..add_start]
                            .iter()
                            .filter_map(|item| {
                                if let FilteredItem::Line {
                                    line,
                                    hunk_line_index,
                                } = item
                                {
                                    Some((*line, *hunk_line_index))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let adds: Vec<_> = items[add_start..i]
                            .iter()
                            .filter_map(|item| {
                                if let FilteredItem::Line {
                                    line,
                                    hunk_line_index,
                                } = item
                                {
                                    Some((*line, *hunk_line_index))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let max = dels.len().max(adds.len());

                        for j in 0..max {
                            let (old_lineno, new_lineno, origin, line_idx) =
                                if j < dels.len() && j < adds.len() {
                                    (
                                        dels[j].0.old_lineno,
                                        adds[j].0.new_lineno,
                                        Some(DiffLineOrigin::Deletion),
                                        Some(dels[j].1),
                                    )
                                } else if j < dels.len() {
                                    (
                                        dels[j].0.old_lineno,
                                        None,
                                        Some(DiffLineOrigin::Deletion),
                                        Some(dels[j].1),
                                    )
                                } else {
                                    (
                                        None,
                                        adds[j].0.new_lineno,
                                        Some(DiffLineOrigin::Addition),
                                        Some(adds[j].1),
                                    )
                                };

                            rows.push(DisplayRowInfo {
                                hunk_index: hunk_idx,
                                line_index: line_idx,
                                old_lineno,
                                new_lineno,
                                origin,
                                is_header: false,
                                is_collapsed_indicator: false,
                                gap_id: None,
                                hidden_count: 0,
                            });
                        }
                    }
                    DiffLineOrigin::Addition => {
                        rows.push(DisplayRowInfo {
                            hunk_index: hunk_idx,
                            line_index: Some(*hunk_line_index),
                            old_lineno: None,
                            new_lineno: line.new_lineno,
                            origin: Some(DiffLineOrigin::Addition),
                            is_header: false,
                            is_collapsed_indicator: false,
                            gap_id: None,
                            hidden_count: 0,
                        });
                        i += 1;
                    }
                },
            }
        }
    }

    rows
}

/// Build a display map for the unified view.
pub fn build_unified_display_map(
    delta: &FileDelta,
    display_context: usize,
    gap_expansions: &HashMap<usize, usize>,
) -> Vec<DisplayRowInfo> {
    let mut rows = Vec::new();
    let mut gap_id_offset = 0;

    for (hunk_idx, hunk) in delta.hunks.iter().enumerate() {
        // Hunk header row
        rows.push(DisplayRowInfo {
            hunk_index: hunk_idx,
            line_index: None,
            old_lineno: None,
            new_lineno: None,
            origin: None,
            is_header: true,
            is_collapsed_indicator: false,
            gap_id: None,
            hidden_count: 0,
        });

        let (items, next_offset) =
            filter_hunk_lines(&hunk.lines, display_context, gap_expansions, gap_id_offset);
        gap_id_offset = next_offset;

        for item in &items {
            match item {
                FilteredItem::CollapsedIndicator {
                    hidden_count,
                    gap_id,
                } => {
                    rows.push(DisplayRowInfo {
                        hunk_index: hunk_idx,
                        line_index: None,
                        old_lineno: None,
                        new_lineno: None,
                        origin: None,
                        is_header: false,
                        is_collapsed_indicator: true,
                        gap_id: Some(*gap_id),
                        hidden_count: *hidden_count,
                    });
                }
                FilteredItem::Line {
                    line,
                    hunk_line_index,
                } => {
                    rows.push(DisplayRowInfo {
                        hunk_index: hunk_idx,
                        line_index: Some(*hunk_line_index),
                        old_lineno: line.old_lineno,
                        new_lineno: line.new_lineno,
                        origin: Some(line.origin.clone()),
                        is_header: false,
                        is_collapsed_indicator: false,
                        gap_id: None,
                        hidden_count: 0,
                    });
                }
            }
        }
    }

    rows
}

/// Build the appropriate display map based on the current view mode.
pub fn build_display_map(
    delta: &FileDelta,
    mode: DiffViewMode,
    display_context: usize,
    gap_expansions: &HashMap<usize, usize>,
) -> Vec<DisplayRowInfo> {
    match mode {
        DiffViewMode::Split => build_split_display_map(delta, display_context, gap_expansions),
        DiffViewMode::Unified => build_unified_display_map(delta, display_context, gap_expansions),
    }
}
