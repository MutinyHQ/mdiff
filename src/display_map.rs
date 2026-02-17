use crate::git::types::{DiffLineOrigin, FileDelta};
use crate::state::DiffViewMode;

/// Information about what a single display row maps to in the diff.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DisplayRowInfo {
    /// Index of the hunk this row belongs to.
    pub hunk_index: usize,
    /// Index of the line within the hunk (None for hunk headers).
    pub line_index: Option<usize>,
    /// Old-side line number (1-based), if applicable.
    pub old_lineno: Option<u32>,
    /// New-side line number (1-based), if applicable.
    pub new_lineno: Option<u32>,
    /// The origin type of this line.
    pub origin: Option<DiffLineOrigin>,
    /// Whether this is a hunk header row.
    pub is_header: bool,
}

/// Build a display map for the split view.
/// Replicates the iteration order of `build_split_lines` in diff_view.rs.
pub fn build_split_display_map(delta: &FileDelta) -> Vec<DisplayRowInfo> {
    let mut rows = Vec::new();

    for (hunk_idx, hunk) in delta.hunks.iter().enumerate() {
        // Hunk header row
        rows.push(DisplayRowInfo {
            hunk_index: hunk_idx,
            line_index: None,
            old_lineno: None,
            new_lineno: None,
            origin: None,
            is_header: true,
        });

        let mut i = 0;
        let lines = &hunk.lines;
        while i < lines.len() {
            match lines[i].origin {
                DiffLineOrigin::Context => {
                    rows.push(DisplayRowInfo {
                        hunk_index: hunk_idx,
                        line_index: Some(i),
                        old_lineno: lines[i].old_lineno,
                        new_lineno: lines[i].new_lineno,
                        origin: Some(DiffLineOrigin::Context),
                        is_header: false,
                    });
                    i += 1;
                }
                DiffLineOrigin::Deletion => {
                    let del_start = i;
                    while i < lines.len() && lines[i].origin == DiffLineOrigin::Deletion {
                        i += 1;
                    }
                    let add_start = i;
                    while i < lines.len() && lines[i].origin == DiffLineOrigin::Addition {
                        i += 1;
                    }
                    let dels = &lines[del_start..add_start];
                    let adds = &lines[add_start..i];
                    let max = dels.len().max(adds.len());

                    for j in 0..max {
                        // In split view, del and add are on the same display row.
                        // We use the deletion line info if available, else the addition.
                        let (old_lineno, new_lineno, origin, line_idx) =
                            if j < dels.len() && j < adds.len() {
                                (
                                    dels[j].old_lineno,
                                    adds[j].new_lineno,
                                    Some(DiffLineOrigin::Deletion),
                                    Some(del_start + j),
                                )
                            } else if j < dels.len() {
                                (
                                    dels[j].old_lineno,
                                    None,
                                    Some(DiffLineOrigin::Deletion),
                                    Some(del_start + j),
                                )
                            } else {
                                (
                                    None,
                                    adds[j].new_lineno,
                                    Some(DiffLineOrigin::Addition),
                                    Some(add_start + j),
                                )
                            };

                        rows.push(DisplayRowInfo {
                            hunk_index: hunk_idx,
                            line_index: line_idx,
                            old_lineno,
                            new_lineno,
                            origin,
                            is_header: false,
                        });
                    }
                }
                DiffLineOrigin::Addition => {
                    rows.push(DisplayRowInfo {
                        hunk_index: hunk_idx,
                        line_index: Some(i),
                        old_lineno: None,
                        new_lineno: lines[i].new_lineno,
                        origin: Some(DiffLineOrigin::Addition),
                        is_header: false,
                    });
                    i += 1;
                }
            }
        }
    }

    rows
}

/// Build a display map for the unified view.
/// Replicates the iteration order of `render_unified` in diff_view.rs.
pub fn build_unified_display_map(delta: &FileDelta) -> Vec<DisplayRowInfo> {
    let mut rows = Vec::new();

    for (hunk_idx, hunk) in delta.hunks.iter().enumerate() {
        // Hunk header row
        rows.push(DisplayRowInfo {
            hunk_index: hunk_idx,
            line_index: None,
            old_lineno: None,
            new_lineno: None,
            origin: None,
            is_header: true,
        });

        for (line_idx, line) in hunk.lines.iter().enumerate() {
            rows.push(DisplayRowInfo {
                hunk_index: hunk_idx,
                line_index: Some(line_idx),
                old_lineno: line.old_lineno,
                new_lineno: line.new_lineno,
                origin: Some(line.origin.clone()),
                is_header: false,
            });
        }
    }

    rows
}

/// Build the appropriate display map based on the current view mode.
pub fn build_display_map(delta: &FileDelta, mode: DiffViewMode) -> Vec<DisplayRowInfo> {
    match mode {
        DiffViewMode::Split => build_split_display_map(delta),
        DiffViewMode::Unified => build_unified_display_map(delta),
    }
}
