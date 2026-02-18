use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::display_map::{
    build_display_map, filter_hunk_lines, DisplayRowInfo, ExpandDirection, FilteredItem,
};
use crate::git::types::{DiffLineOrigin, FileDelta};
use crate::highlight::HighlightSpan;
use crate::state::{app_state::FocusPanel, AppState, DiffViewMode};
use crate::theme::Theme;

use super::Component;

pub struct DiffView;

impl Component for DiffView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focus == FocusPanel::DiffView;
        let theme = &state.theme;

        let border_style = if is_focused {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let view_label = match state.diff.options.view_mode {
            DiffViewMode::Split => "Split",
            DiffViewMode::Unified => "Unified",
        };

        let Some(delta) = state.diff.selected_delta() else {
            let block = Block::default()
                .title(format!(" Diff [{view_label}] "))
                .borders(Borders::ALL)
                .border_style(border_style);

            let content = if state.diff.loading {
                " Loading..."
            } else if state.diff.deltas.is_empty() {
                " No changes detected"
            } else {
                " Select a file to view diff"
            };

            let paragraph = Paragraph::new(content)
                .style(Style::default().fg(theme.text_muted))
                .block(block);
            frame.render_widget(paragraph, area);
            return;
        };

        match state.diff.options.view_mode {
            DiffViewMode::Split => {
                render_split(frame, area, delta, state, border_style, view_label, theme)
            }
            DiffViewMode::Unified => {
                render_unified(frame, area, delta, state, border_style, view_label, theme)
            }
        }
    }
}

fn format_title(delta: &FileDelta, view_label: &str, state: &AppState) -> String {
    let path_display = delta.path.to_string_lossy();
    let base = if let Some(ref old_path) = delta.old_path {
        if *old_path != delta.path {
            let old_display = old_path.to_string_lossy();
            format!(" {old_display} \u{2192} {path_display} [{view_label}]")
        } else {
            format!(" {path_display} [{view_label}]")
        }
    } else {
        format!(" {path_display} [{view_label}]")
    };

    if state.diff.search_active || !state.diff.search_query.is_empty() {
        let match_info = if state.diff.search_matches.is_empty() {
            if state.diff.search_query.is_empty() {
                String::new()
            } else {
                " (no matches)".to_string()
            }
        } else {
            let idx = state.diff.search_match_index.map(|i| i + 1).unwrap_or(0);
            format!(" ({}/{})", idx, state.diff.search_matches.len())
        };
        format!("{base} /{}{match_info} ", state.diff.search_query)
    } else {
        format!("{base} ")
    }
}

/// Check if a display row index is within the current visual selection range.
fn is_row_selected(state: &AppState, display_row: usize) -> bool {
    if !state.selection.active {
        return false;
    }
    let (start, end) = state.selection.range();
    display_row >= start && display_row <= end
}

/// Check if a display row is the cursor row (when not in visual selection mode).
fn is_cursor_row(state: &AppState, display_row: usize) -> bool {
    !state.selection.active
        && state.focus == FocusPanel::DiffView
        && display_row == state.diff.cursor_row
}

/// Per-row highlight: separates gutter indicator from content background.
#[derive(Clone, Copy, Default)]
struct RowHighlight {
    /// Background for the gutter/line-number area.
    gutter_bg: Option<Color>,
    /// Foreground for the gutter (overrides text_muted when set).
    gutter_fg: Option<Color>,
    /// Background override for content area. When set, replaces diff_bg.
    content_bg: Option<Color>,
}

/// Check if a display row is a search match.
fn is_search_match(state: &AppState, display_row: usize) -> bool {
    !state.diff.search_query.is_empty()
        && state
            .diff
            .search_matches
            .binary_search(&display_row)
            .is_ok()
}

/// Compute row highlight for cursor or visual selection.
fn row_highlight(state: &AppState, display_row: usize) -> RowHighlight {
    let theme = &state.theme;
    if is_row_selected(state, display_row) {
        let bg = Some(theme.visual_select_bg);
        RowHighlight {
            gutter_bg: bg,
            gutter_fg: None,
            content_bg: bg,
        }
    } else if is_cursor_row(state, display_row) {
        RowHighlight {
            gutter_bg: Some(theme.accent),
            gutter_fg: Some(Color::Black),
            content_bg: None,
        }
    } else if is_search_match(state, display_row) {
        RowHighlight {
            gutter_bg: Some(theme.warning),
            gutter_fg: Some(Color::Black),
            content_bg: Some(theme.visual_select_bg),
        }
    } else {
        RowHighlight::default()
    }
}

/// Check if a line has an annotation marker in the gutter.
fn has_annotation(state: &AppState, delta: &FileDelta, row_info: &DisplayRowInfo) -> bool {
    let file_path = delta.path.to_string_lossy();
    // Check both old and new line numbers
    if let Some(n) = row_info.new_lineno {
        if state.annotations.has_annotation_at(&file_path, n) {
            return true;
        }
    }
    if let Some(n) = row_info.old_lineno {
        if state.annotations.has_annotation_at(&file_path, n) {
            return true;
        }
    }
    false
}

fn render_split(
    frame: &mut Frame,
    area: Rect,
    delta: &FileDelta,
    state: &AppState,
    border_style: Style,
    view_label: &str,
    theme: &Theme,
) {
    let title = format_title(delta, view_label, state);

    if delta.binary {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        let msg = Paragraph::new(" Binary file differs")
            .style(Style::default().fg(theme.text_muted))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let old_hl = &state.diff.old_highlights;
    let new_hl = &state.diff.new_highlights;

    // Build display map for selection/annotation checking
    let display_map = build_display_map(
        delta,
        DiffViewMode::Split,
        state.diff.display_context,
        &state.diff.gap_expansions,
    );

    let (left_lines, right_lines) = build_split_lines(
        delta,
        state.diff.scroll_offset,
        inner.height as usize,
        old_hl,
        new_hl,
        state,
        &display_map,
        halves[0].width,
        true,
        theme,
    );

    let left_para = Paragraph::new(left_lines);
    let right_para = Paragraph::new(right_lines);

    frame.render_widget(left_para, halves[0]);
    frame.render_widget(right_para, halves[1]);
}

#[allow(clippy::too_many_arguments)]
fn build_split_lines<'a>(
    delta: &'a FileDelta,
    scroll: usize,
    height: usize,
    old_hl: &[Vec<HighlightSpan>],
    new_hl: &[Vec<HighlightSpan>],
    state: &AppState,
    display_map: &[DisplayRowInfo],
    width: u16,
    wrap_enabled: bool,
    theme: &Theme,
) -> (Vec<Line<'a>>, Vec<Line<'a>>) {
    let mut left: Vec<Line> = Vec::new();
    let mut right: Vec<Line> = Vec::new();
    let mut display_row: usize = 0;

    let gutter_width = 5;
    let mut gap_id_offset = 0;

    for hunk in &delta.hunks {
        let hl = row_highlight(state, display_row);
        let ann_marker = display_map
            .get(display_row)
            .is_some_and(|info| has_annotation(state, delta, info));

        left.push(make_hunk_header_line(
            gutter_width,
            &hunk.header,
            hl,
            ann_marker,
            theme,
        ));
        right.push(make_hunk_header_line(
            gutter_width,
            &hunk.header,
            hl,
            false,
            theme,
        ));
        display_row += 1;

        let (items, next_offset) = filter_hunk_lines(
            &hunk.lines,
            state.diff.display_context,
            &state.diff.gap_expansions,
            gap_id_offset,
        );
        gap_id_offset = next_offset;

        let mut i = 0;
        while i < items.len() {
            match &items[i] {
                FilteredItem::CollapsedIndicator {
                    hidden_count,
                    direction,
                    ..
                } => {
                    let hl = row_highlight(state, display_row);
                    left.push(make_collapsed_indicator_line(
                        gutter_width,
                        *hidden_count,
                        *direction,
                        hl,
                        theme,
                    ));
                    right.push(make_collapsed_indicator_line(
                        gutter_width,
                        *hidden_count,
                        *direction,
                        hl,
                        theme,
                    ));
                    display_row += 1;
                    i += 1;
                }
                FilteredItem::Line { line, .. } => match line.origin {
                    DiffLineOrigin::Context => {
                        let hl = row_highlight(state, display_row);
                        let ann_marker = display_map
                            .get(display_row)
                            .is_some_and(|info| has_annotation(state, delta, info));

                        let gutter_l = format_lineno(line.old_lineno, gutter_width);
                        let gutter_r = format_lineno(line.new_lineno, gutter_width);
                        let old_spans = line.old_lineno.and_then(|n| old_hl.get(n as usize));
                        let new_spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                        left.push(make_highlighted_line(
                            &gutter_l,
                            &line.content,
                            old_spans,
                            None,
                            hl,
                            ann_marker,
                            theme,
                        ));
                        right.push(make_highlighted_line(
                            &gutter_r,
                            &line.content,
                            new_spans,
                            None,
                            hl,
                            false,
                            theme,
                        ));
                        display_row += 1;
                        i += 1;
                    }
                    DiffLineOrigin::Deletion => {
                        // Collect consecutive deletions from filtered items
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
                                if let FilteredItem::Line { line, .. } = item {
                                    Some(*line)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let adds: Vec<_> = items[add_start..i]
                            .iter()
                            .filter_map(|item| {
                                if let FilteredItem::Line { line, .. } = item {
                                    Some(*line)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let max = dels.len().max(adds.len());

                        for j in 0..max {
                            let hl = row_highlight(state, display_row);
                            let ann_marker = display_map
                                .get(display_row)
                                .is_some_and(|info| has_annotation(state, delta, info));

                            if j < dels.len() {
                                let line = dels[j];
                                let gutter = format_lineno(line.old_lineno, gutter_width);
                                let spans = line.old_lineno.and_then(|n| old_hl.get(n as usize));
                                left.push(make_highlighted_line(
                                    &gutter,
                                    &line.content,
                                    spans,
                                    Some(theme.diff_del_bg),
                                    hl,
                                    ann_marker,
                                    theme,
                                ));
                            } else {
                                left.push(make_empty_line(gutter_width, hl, theme));
                            }

                            if j < adds.len() {
                                let line = adds[j];
                                let gutter = format_lineno(line.new_lineno, gutter_width);
                                let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                                right.push(make_highlighted_line(
                                    &gutter,
                                    &line.content,
                                    spans,
                                    Some(theme.diff_add_bg),
                                    hl,
                                    false,
                                    theme,
                                ));
                            } else {
                                right.push(make_empty_line(gutter_width, hl, theme));
                            }

                            display_row += 1;
                        }
                    }
                    DiffLineOrigin::Addition => {
                        let hl = row_highlight(state, display_row);
                        let ann_marker = display_map
                            .get(display_row)
                            .is_some_and(|info| has_annotation(state, delta, info));

                        let gutter = format_lineno(line.new_lineno, gutter_width);
                        let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                        left.push(make_empty_line(gutter_width, hl, theme));
                        right.push(make_highlighted_line(
                            &gutter,
                            &line.content,
                            spans,
                            Some(theme.diff_add_bg),
                            hl,
                            ann_marker,
                            theme,
                        ));
                        display_row += 1;
                        i += 1;
                    }
                },
            }
        }
    }

    let left_visible: Vec<Line> = left.into_iter().skip(scroll).take(height).collect();
    let right_visible: Vec<Line> = right.into_iter().skip(scroll).take(height).collect();

    (
        wrap_lines_for_display(left_visible, width, gutter_width + 1, wrap_enabled, theme),
        wrap_lines_for_display(right_visible, width, gutter_width + 1, wrap_enabled, theme),
    )
}

fn render_unified(
    frame: &mut Frame,
    area: Rect,
    delta: &FileDelta,
    state: &AppState,
    border_style: Style,
    view_label: &str,
    theme: &Theme,
) {
    let title = format_title(delta, view_label, state);

    if delta.binary {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        let msg = Paragraph::new(" Binary file differs")
            .style(Style::default().fg(theme.text_muted))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let old_hl = &state.diff.old_highlights;
    let new_hl = &state.diff.new_highlights;

    // Build display map for selection/annotation checking
    let display_map = build_display_map(
        delta,
        DiffViewMode::Unified,
        state.diff.display_context,
        &state.diff.gap_expansions,
    );

    let gutter_width = 5;
    let mut lines: Vec<Line> = Vec::new();
    let mut display_row: usize = 0;
    let mut gap_id_offset = 0;

    for hunk in &delta.hunks {
        let hl = row_highlight(state, display_row);
        let ann_marker = display_map
            .get(display_row)
            .is_some_and(|info| has_annotation(state, delta, info));

        lines.push(make_hunk_header_line_unified(
            gutter_width,
            &hunk.header,
            hl,
            ann_marker,
            theme,
        ));
        display_row += 1;

        let (items, next_offset) = filter_hunk_lines(
            &hunk.lines,
            state.diff.display_context,
            &state.diff.gap_expansions,
            gap_id_offset,
        );
        gap_id_offset = next_offset;

        for item in &items {
            match item {
                FilteredItem::CollapsedIndicator {
                    hidden_count,
                    direction,
                    ..
                } => {
                    let hl = row_highlight(state, display_row);
                    lines.push(make_collapsed_indicator_line_unified(
                        gutter_width,
                        *hidden_count,
                        *direction,
                        hl,
                        theme,
                    ));
                    display_row += 1;
                }
                FilteredItem::Line { line, .. } => {
                    let hl = row_highlight(state, display_row);
                    let ann_marker = display_map
                        .get(display_row)
                        .is_some_and(|info| has_annotation(state, delta, info));

                    let (old_g, new_g) = (
                        format_lineno(line.old_lineno, gutter_width),
                        format_lineno(line.new_lineno, gutter_width),
                    );

                    match line.origin {
                        DiffLineOrigin::Context => {
                            let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                            lines.push(make_unified_highlighted(
                                &old_g,
                                &new_g,
                                " ",
                                &line.content,
                                spans,
                                None,
                                hl,
                                ann_marker,
                                theme,
                            ));
                        }
                        DiffLineOrigin::Addition => {
                            let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                            let blank = " ".repeat(gutter_width);
                            lines.push(make_unified_highlighted(
                                &blank,
                                &new_g,
                                "+",
                                &line.content,
                                spans,
                                Some(theme.diff_add_bg),
                                hl,
                                ann_marker,
                                theme,
                            ));
                        }
                        DiffLineOrigin::Deletion => {
                            let spans = line.old_lineno.and_then(|n| old_hl.get(n as usize));
                            let blank = " ".repeat(gutter_width);
                            lines.push(make_unified_highlighted(
                                &old_g,
                                &blank,
                                "-",
                                &line.content,
                                spans,
                                Some(theme.diff_del_bg),
                                hl,
                                ann_marker,
                                theme,
                            ));
                        }
                    }
                    display_row += 1;
                }
            }
        }
    }

    let visible: Vec<Line> = lines
        .into_iter()
        .skip(state.diff.scroll_offset)
        .take(inner.height as usize)
        .collect();
    // Unified gutter: old_lineno(5) + space(1) + new_lineno(5) + marker(1) + prefix(1) = 13
    let unified_gutter_width = gutter_width + 1 + gutter_width + 1 + 1;
    let wrapped = wrap_lines_for_display(visible, inner.width, unified_gutter_width, true, theme);
    let paragraph = Paragraph::new(wrapped);
    frame.render_widget(paragraph, inner);
}

// Helper functions

fn format_lineno(lineno: Option<u32>, width: usize) -> String {
    match lineno {
        Some(n) => format!("{n:>width$}"),
        None => " ".repeat(width),
    }
}

/// Format a gutter string, optionally replacing the trailing space with an annotation marker.
fn format_gutter_with_marker(gutter: &str, ann_marker: bool) -> String {
    if ann_marker {
        format!("{gutter}\u{2502}")
    } else {
        format!("{gutter} ")
    }
}

/// Build a hunk header line for split view.
fn make_hunk_header_line<'a>(
    gutter_width: usize,
    header: &str,
    hl: RowHighlight,
    ann_marker: bool,
    theme: &Theme,
) -> Line<'a> {
    let marker = if ann_marker { "\u{2502}" } else { " " };
    let gutter_text = format!("{:>gutter_width$}{marker}", "...");
    let mut gutter_style = Style::default().fg(theme.text_muted);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default().fg(theme.text_muted);
    if let Some(bg) = hl.content_bg {
        content_style = content_style.bg(bg);
    }
    Line::from(vec![
        Span::styled(gutter_text, gutter_style),
        Span::styled(header.to_string(), content_style),
    ])
}

/// Build a hunk header line for unified view.
fn make_hunk_header_line_unified<'a>(
    gutter_width: usize,
    header: &str,
    hl: RowHighlight,
    ann_marker: bool,
    theme: &Theme,
) -> Line<'a> {
    let marker = if ann_marker { "\u{2502}" } else { " " };
    let gutter_text = format!("{:>gutter_width$}{marker}", "...");
    let mut gutter_style = Style::default().fg(theme.text_muted);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default()
        .fg(theme.diff_hunk_header_fg)
        .add_modifier(Modifier::BOLD);
    if let Some(bg) = hl.content_bg {
        content_style = content_style.bg(bg);
    }
    Line::from(vec![
        Span::styled(gutter_text, gutter_style),
        Span::styled(header.to_string(), content_style),
    ])
}

/// Build a line with syntax highlighting spans overlaid on a diff background.
/// Gutter and content are highlighted separately via RowHighlight.
fn make_highlighted_line<'a>(
    gutter: &str,
    content: &str,
    hl_spans: Option<&Vec<HighlightSpan>>,
    diff_bg: Option<Color>,
    hl: RowHighlight,
    ann_marker: bool,
    theme: &Theme,
) -> Line<'a> {
    let trimmed = content.trim_end_matches('\n');
    let content_bg = hl.content_bg.or(diff_bg);
    let gutter_text = format_gutter_with_marker(gutter, ann_marker);

    let mut gutter_style = Style::default().fg(theme.text_muted);
    if ann_marker {
        gutter_style = gutter_style.fg(theme.cursor_line_fg);
    }
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let gutter_span = Span::styled(gutter_text, gutter_style);

    let content_spans = if let Some(spans) = hl_spans {
        apply_highlights(trimmed, spans, content_bg, theme)
    } else {
        // Fallback: no highlighting
        let mut style = Style::default();
        if let Some(bg_color) = content_bg {
            style = style.bg(bg_color);
            if hl.content_bg.is_none() {
                // Use diff-specific fg colors only when not selected
                if bg_color == theme.diff_del_bg {
                    style = style.fg(theme.diff_del_fg);
                } else if bg_color == theme.diff_add_bg {
                    style = style.fg(theme.diff_add_fg);
                } else {
                    style = style.fg(theme.text);
                }
            } else {
                style = style.fg(theme.text);
            }
        } else {
            style = style.fg(theme.text);
        }
        vec![Span::styled(trimmed.to_string(), style)]
    };

    let mut all_spans = vec![gutter_span];
    all_spans.extend(content_spans);
    Line::from(all_spans)
}

/// Apply highlight spans to a string, blending with diff background.
fn apply_highlights<'a>(
    text: &str,
    hl_spans: &[HighlightSpan],
    bg: Option<Color>,
    theme: &Theme,
) -> Vec<Span<'a>> {
    if hl_spans.is_empty() || text.is_empty() {
        let mut style = Style::default().fg(theme.text);
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
        }
        return vec![Span::styled(text.to_string(), style)];
    }

    let mut result = Vec::new();
    let mut pos = 0;
    let text_len = text.len();

    for span in hl_spans {
        let start = span.start.min(text_len);
        let end = span.end.min(text_len);

        if start > pos {
            // Gap before this span — use default style
            let mut style = Style::default().fg(theme.diff_context_fg);
            if let Some(bg_color) = bg {
                style = style.bg(bg_color);
            }
            result.push(Span::styled(text[pos..start].to_string(), style));
        }

        if start < end {
            let mut style = span.style;
            if let Some(bg_color) = bg {
                style = style.bg(bg_color);
            }
            result.push(Span::styled(text[start..end].to_string(), style));
        }

        pos = end;
    }

    // Remaining text after last span
    if pos < text_len {
        let mut style = Style::default().fg(theme.diff_context_fg);
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
        }
        result.push(Span::styled(text[pos..].to_string(), style));
    }

    result
}

fn make_empty_line<'a>(gutter_width: usize, hl: RowHighlight, theme: &Theme) -> Line<'a> {
    let mut gutter_style = Style::default().fg(theme.text_muted).bg(theme.collapsed_bg);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default().fg(theme.text_muted).bg(theme.collapsed_bg);
    if let Some(bg) = hl.content_bg {
        content_style = content_style.bg(bg);
    }
    Line::from(vec![
        Span::styled(format!("{} ", " ".repeat(gutter_width)), gutter_style),
        Span::styled(" ", content_style),
    ])
}

#[allow(clippy::too_many_arguments)]
fn make_unified_highlighted<'a>(
    old_g: &str,
    new_g: &str,
    prefix: &str,
    content: &str,
    hl_spans: Option<&Vec<HighlightSpan>>,
    diff_bg: Option<Color>,
    hl: RowHighlight,
    ann_marker: bool,
    theme: &Theme,
) -> Line<'a> {
    let trimmed = content.trim_end_matches('\n');
    let content_bg = hl.content_bg.or(diff_bg);

    let marker = if ann_marker { "\u{2502}" } else { " " };
    let mut gutter_style = Style::default().fg(theme.text_muted);
    if ann_marker {
        gutter_style = gutter_style.fg(theme.cursor_line_fg);
    }
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let gutter_span = Span::styled(format!("{old_g} {new_g}{marker}"), gutter_style);

    let prefix_style = match prefix {
        "+" => Style::default()
            .fg(theme.diff_add_fg)
            .bg(content_bg.unwrap_or_default()),
        "-" => Style::default()
            .fg(theme.diff_del_fg)
            .bg(content_bg.unwrap_or_default()),
        _ => {
            let mut s = Style::default().fg(theme.text_muted);
            if let Some(bg_color) = content_bg {
                s = s.bg(bg_color);
            }
            s
        }
    };
    let prefix_span = Span::styled(prefix.to_string(), prefix_style);

    let content_spans = if let Some(spans) = hl_spans {
        apply_highlights(trimmed, spans, content_bg, theme)
    } else {
        let mut style = Style::default().fg(theme.text);
        if let Some(bg_color) = content_bg {
            style = style.bg(bg_color);
        }
        vec![Span::styled(trimmed.to_string(), style)]
    };

    let mut all_spans = vec![gutter_span, prefix_span];
    all_spans.extend(content_spans);
    Line::from(all_spans)
}

/// Build a collapsed indicator line for split view.
fn make_collapsed_indicator_line<'a>(
    gutter_width: usize,
    hidden_count: usize,
    direction: ExpandDirection,
    hl: RowHighlight,
    theme: &Theme,
) -> Line<'a> {
    let gutter_text = format!("{:>gutter_width$} ", "\u{22ef}");
    let mut gutter_style = Style::default().fg(theme.text_muted);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default().fg(theme.text_muted);
    if let Some(bg) = hl.content_bg {
        content_style = content_style.bg(bg);
    }
    let caret = match direction {
        ExpandDirection::Down => "\u{25bc}", // ▼
        ExpandDirection::Up => "\u{25b2}",   // ▲
    };
    let label = format!("{caret} {hidden_count} lines hidden {caret}");
    Line::from(vec![
        Span::styled(gutter_text, gutter_style),
        Span::styled(label, content_style),
    ])
}

/// Build a collapsed indicator line for unified view.
fn make_collapsed_indicator_line_unified<'a>(
    gutter_width: usize,
    hidden_count: usize,
    direction: ExpandDirection,
    hl: RowHighlight,
    theme: &Theme,
) -> Line<'a> {
    let gutter_text = format!(
        "{:>gutter_width$} {:>gutter_width$} ",
        "\u{22ef}", "\u{22ef}"
    );
    let mut gutter_style = Style::default().fg(theme.text_muted);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default().fg(theme.text_muted);
    if let Some(bg) = hl.content_bg {
        content_style = content_style.bg(bg);
    }
    let caret = match direction {
        ExpandDirection::Down => "\u{25bc}", // ▼
        ExpandDirection::Up => "\u{25b2}",   // ▲
    };
    let label = format!("{caret} {hidden_count} lines hidden {caret}");
    Line::from(vec![
        Span::styled(gutter_text, gutter_style),
        Span::styled(label, content_style),
    ])
}

/// Wrap lines that exceed the available width, producing continuation lines with a ↪ gutter.
/// `gutter_width` is the total width of the gutter area (line numbers + marker).
/// Content spans (all spans after the first gutter span) are split across visual lines.
fn wrap_lines_for_display<'a>(
    lines: Vec<Line<'a>>,
    width: u16,
    gutter_width: usize,
    wrap_enabled: bool,
    theme: &Theme,
) -> Vec<Line<'a>> {
    if !wrap_enabled || width == 0 {
        return lines;
    }
    let max_width = width as usize;
    let content_width = max_width.saturating_sub(gutter_width);
    if content_width == 0 {
        return lines;
    }

    let mut result: Vec<Line<'a>> = Vec::new();

    for line in lines {
        let line_width: usize = line.spans.iter().map(|s| s.width()).sum();
        if line_width <= max_width {
            result.push(line);
            continue;
        }

        // Separate gutter (first span) from content (remaining spans)
        let mut spans_iter = line.spans.into_iter();
        let gutter_span = match spans_iter.next() {
            Some(s) => s,
            None => {
                result.push(Line::default());
                continue;
            }
        };
        let content_spans: Vec<Span<'a>> = spans_iter.collect();

        // Flatten content spans into (char, Style) pairs for splitting
        let mut chars: Vec<(char, Style)> = Vec::new();
        for span in &content_spans {
            let style = span.style;
            for ch in span.content.chars() {
                chars.push((ch, style));
            }
        }

        // Build the continuation gutter
        let cont_gutter = format!("{}\u{21aa}", " ".repeat(gutter_width.saturating_sub(1)));
        let cont_gutter_style = Style::default().fg(theme.text_muted);

        // Split chars into chunks of content_width
        let mut offset = 0;
        let mut is_first = true;
        while offset < chars.len() {
            let end = (offset + content_width).min(chars.len());
            let chunk = &chars[offset..end];

            // Build spans from the chunk, coalescing adjacent chars with the same style
            let mut chunk_spans: Vec<Span<'a>> = Vec::new();
            let mut current_text = String::new();
            let mut current_style = chunk[0].1;

            for &(ch, style) in chunk {
                if style == current_style {
                    current_text.push(ch);
                } else {
                    if !current_text.is_empty() {
                        chunk_spans.push(Span::styled(current_text, current_style));
                        current_text = String::new();
                    }
                    current_style = style;
                    current_text.push(ch);
                }
            }
            if !current_text.is_empty() {
                chunk_spans.push(Span::styled(current_text, current_style));
            }

            let mut line_spans = Vec::new();
            if is_first {
                line_spans.push(gutter_span.clone());
                is_first = false;
            } else {
                line_spans.push(Span::styled(cont_gutter.clone(), cont_gutter_style));
            }
            line_spans.extend(chunk_spans);
            result.push(Line::from(line_spans));

            offset = end;
        }

        // Edge case: if content was empty but gutter was wide
        if is_first {
            result.push(Line::from(vec![gutter_span]));
        }
    }

    result
}
