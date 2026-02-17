use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::display_map::{build_display_map, DisplayRowInfo};
use crate::git::types::{DiffLineOrigin, FileDelta};
use crate::highlight::HighlightSpan;
use crate::state::{app_state::FocusPanel, AppState, DiffViewMode};

use super::Component;

pub struct DiffView;

impl Component for DiffView {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focus == FocusPanel::DiffView;

        let border_style = if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
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
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(paragraph, area);
            return;
        };

        match state.diff.options.view_mode {
            DiffViewMode::Split => {
                render_split(frame, area, delta, state, border_style, view_label)
            }
            DiffViewMode::Unified => {
                render_unified(frame, area, delta, state, border_style, view_label)
            }
        }
    }
}

fn format_title(delta: &FileDelta, view_label: &str) -> String {
    let path_display = delta.path.to_string_lossy();
    if let Some(ref old_path) = delta.old_path {
        if *old_path != delta.path {
            let old_display = old_path.to_string_lossy();
            return format!(" {old_display} \u{2192} {path_display} [{view_label}] ");
        }
    }
    format!(" {path_display} [{view_label}] ")
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
    /// Foreground for the gutter (overrides DarkGray when set).
    gutter_fg: Option<Color>,
    /// Background override for content area. When set, replaces diff_bg.
    content_bg: Option<Color>,
}

/// Compute row highlight for cursor or visual selection.
fn row_highlight(state: &AppState, display_row: usize) -> RowHighlight {
    if is_row_selected(state, display_row) {
        let bg = Some(Color::Rgb(70, 50, 100));
        RowHighlight {
            gutter_bg: bg,
            gutter_fg: None,
            content_bg: bg,
        }
    } else if is_cursor_row(state, display_row) {
        RowHighlight {
            gutter_bg: Some(Color::Cyan),
            gutter_fg: Some(Color::Black),
            content_bg: None,
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
) {
    let title = format_title(delta, view_label);

    if delta.binary {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        let msg = Paragraph::new(" Binary file differs")
            .style(Style::default().fg(Color::DarkGray))
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
    let display_map = build_display_map(delta, DiffViewMode::Split);

    let (left_lines, right_lines) = build_split_lines(
        delta,
        state.diff.scroll_offset,
        inner.height as usize,
        old_hl,
        new_hl,
        state,
        &display_map,
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
) -> (Vec<Line<'a>>, Vec<Line<'a>>) {
    let mut left: Vec<Line> = Vec::new();
    let mut right: Vec<Line> = Vec::new();
    let mut display_row: usize = 0;

    let gutter_width = 5;

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
        ));
        right.push(make_hunk_header_line(gutter_width, &hunk.header, hl, false));
        display_row += 1;

        let mut i = 0;
        let lines = &hunk.lines;
        while i < lines.len() {
            match lines[i].origin {
                DiffLineOrigin::Context => {
                    let line = &lines[i];
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
                    ));
                    right.push(make_highlighted_line(
                        &gutter_r,
                        &line.content,
                        new_spans,
                        None,
                        hl,
                        false,
                    ));
                    display_row += 1;
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
                        let hl = row_highlight(state, display_row);
                        let ann_marker = display_map
                            .get(display_row)
                            .is_some_and(|info| has_annotation(state, delta, info));

                        if j < dels.len() {
                            let line = &dels[j];
                            let gutter = format_lineno(line.old_lineno, gutter_width);
                            let spans = line.old_lineno.and_then(|n| old_hl.get(n as usize));
                            left.push(make_highlighted_line(
                                &gutter,
                                &line.content,
                                spans,
                                Some(Color::Rgb(40, 0, 0)),
                                hl,
                                ann_marker,
                            ));
                        } else {
                            left.push(make_empty_line(gutter_width, hl));
                        }

                        if j < adds.len() {
                            let line = &adds[j];
                            let gutter = format_lineno(line.new_lineno, gutter_width);
                            let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                            right.push(make_highlighted_line(
                                &gutter,
                                &line.content,
                                spans,
                                Some(Color::Rgb(0, 30, 0)),
                                hl,
                                false,
                            ));
                        } else {
                            right.push(make_empty_line(gutter_width, hl));
                        }

                        display_row += 1;
                    }
                }
                DiffLineOrigin::Addition => {
                    let line = &lines[i];
                    let hl = row_highlight(state, display_row);
                    let ann_marker = display_map
                        .get(display_row)
                        .is_some_and(|info| has_annotation(state, delta, info));

                    let gutter = format_lineno(line.new_lineno, gutter_width);
                    let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                    left.push(make_empty_line(gutter_width, hl));
                    right.push(make_highlighted_line(
                        &gutter,
                        &line.content,
                        spans,
                        Some(Color::Rgb(0, 30, 0)),
                        hl,
                        ann_marker,
                    ));
                    display_row += 1;
                    i += 1;
                }
            }
        }
    }

    let left_visible: Vec<Line> = left.into_iter().skip(scroll).take(height).collect();
    let right_visible: Vec<Line> = right.into_iter().skip(scroll).take(height).collect();

    (left_visible, right_visible)
}

fn render_unified(
    frame: &mut Frame,
    area: Rect,
    delta: &FileDelta,
    state: &AppState,
    border_style: Style,
    view_label: &str,
) {
    let title = format_title(delta, view_label);

    if delta.binary {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        let msg = Paragraph::new(" Binary file differs")
            .style(Style::default().fg(Color::DarkGray))
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
    let display_map = build_display_map(delta, DiffViewMode::Unified);

    let gutter_width = 5;
    let mut lines: Vec<Line> = Vec::new();
    let mut display_row: usize = 0;

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
        ));
        display_row += 1;

        for line in &hunk.lines {
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
                        Some(Color::Rgb(0, 30, 0)),
                        hl,
                        ann_marker,
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
                        Some(Color::Rgb(40, 0, 0)),
                        hl,
                        ann_marker,
                    ));
                }
            }
            display_row += 1;
        }
    }

    let visible: Vec<Line> = lines
        .into_iter()
        .skip(state.diff.scroll_offset)
        .take(inner.height as usize)
        .collect();
    let paragraph = Paragraph::new(visible);
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
) -> Line<'a> {
    let marker = if ann_marker { "\u{2502}" } else { " " };
    let gutter_text = format!("{:>gutter_width$}{marker}", "...");
    let mut gutter_style = Style::default().fg(Color::DarkGray);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default().fg(Color::DarkGray);
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
) -> Line<'a> {
    let marker = if ann_marker { "\u{2502}" } else { " " };
    let gutter_text = format!("{:>gutter_width$}{marker}", "...");
    let mut gutter_style = Style::default().fg(Color::DarkGray);
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default()
        .fg(Color::Magenta)
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
) -> Line<'a> {
    let trimmed = content.trim_end_matches('\n');
    let content_bg = hl.content_bg.or(diff_bg);
    let gutter_text = format_gutter_with_marker(gutter, ann_marker);

    let mut gutter_style = Style::default().fg(Color::DarkGray);
    if ann_marker {
        gutter_style = gutter_style.fg(Color::Yellow);
    }
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let gutter_span = Span::styled(gutter_text, gutter_style);

    let content_spans = if let Some(spans) = hl_spans {
        apply_highlights(trimmed, spans, content_bg)
    } else {
        // Fallback: no highlighting
        let mut style = Style::default();
        if let Some(bg_color) = content_bg {
            style = style.bg(bg_color);
            if hl.content_bg.is_none() {
                // Use diff-specific fg colors only when not selected
                if bg_color == Color::Rgb(40, 0, 0) {
                    style = style.fg(Color::Red);
                } else if bg_color == Color::Rgb(0, 30, 0) {
                    style = style.fg(Color::Green);
                } else {
                    style = style.fg(Color::White);
                }
            } else {
                style = style.fg(Color::White);
            }
        } else {
            style = style.fg(Color::White);
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
) -> Vec<Span<'a>> {
    if hl_spans.is_empty() || text.is_empty() {
        let mut style = Style::default().fg(Color::White);
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
            let mut style = Style::default().fg(Color::Rgb(171, 178, 191));
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
        let mut style = Style::default().fg(Color::Rgb(171, 178, 191));
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
        }
        result.push(Span::styled(text[pos..].to_string(), style));
    }

    result
}

fn make_empty_line<'a>(gutter_width: usize, hl: RowHighlight) -> Line<'a> {
    let mut gutter_style = Style::default()
        .fg(Color::DarkGray)
        .bg(Color::Rgb(20, 20, 20));
    if let Some(fg) = hl.gutter_fg {
        gutter_style = gutter_style.fg(fg);
    }
    if let Some(bg) = hl.gutter_bg {
        gutter_style = gutter_style.bg(bg);
    }
    let mut content_style = Style::default()
        .fg(Color::DarkGray)
        .bg(Color::Rgb(20, 20, 20));
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
) -> Line<'a> {
    let trimmed = content.trim_end_matches('\n');
    let content_bg = hl.content_bg.or(diff_bg);

    let marker = if ann_marker { "\u{2502}" } else { " " };
    let mut gutter_style = Style::default().fg(Color::DarkGray);
    if ann_marker {
        gutter_style = gutter_style.fg(Color::Yellow);
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
            .fg(Color::Green)
            .bg(content_bg.unwrap_or_default()),
        "-" => Style::default()
            .fg(Color::Red)
            .bg(content_bg.unwrap_or_default()),
        _ => {
            let mut s = Style::default().fg(Color::DarkGray);
            if let Some(bg_color) = content_bg {
                s = s.bg(bg_color);
            }
            s
        }
    };
    let prefix_span = Span::styled(prefix.to_string(), prefix_style);

    let content_spans = if let Some(spans) = hl_spans {
        apply_highlights(trimmed, spans, content_bg)
    } else {
        let mut style = Style::default().fg(Color::White);
        if let Some(bg_color) = content_bg {
            style = style.bg(bg_color);
        }
        vec![Span::styled(trimmed.to_string(), style)]
    };

    let mut all_spans = vec![gutter_span, prefix_span];
    all_spans.extend(content_spans);
    Line::from(all_spans)
}
