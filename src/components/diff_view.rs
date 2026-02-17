use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

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

    let (left_lines, right_lines) = build_split_lines(
        delta,
        state.diff.scroll_offset,
        inner.height as usize,
        old_hl,
        new_hl,
    );

    let left_para = Paragraph::new(left_lines);
    let right_para = Paragraph::new(right_lines);

    frame.render_widget(left_para, halves[0]);
    frame.render_widget(right_para, halves[1]);
}

fn build_split_lines<'a>(
    delta: &'a FileDelta,
    scroll: usize,
    height: usize,
    old_hl: &[Vec<HighlightSpan>],
    new_hl: &[Vec<HighlightSpan>],
) -> (Vec<Line<'a>>, Vec<Line<'a>>) {
    let mut left: Vec<Line> = Vec::new();
    let mut right: Vec<Line> = Vec::new();

    let gutter_width = 5;

    for hunk in &delta.hunks {
        left.push(Line::from(Span::styled(
            format!("{:>gutter_width$} {}", "...", &hunk.header),
            Style::default().fg(Color::DarkGray),
        )));
        right.push(Line::from(Span::styled(
            format!("{:>gutter_width$} {}", "...", &hunk.header),
            Style::default().fg(Color::DarkGray),
        )));

        let mut i = 0;
        let lines = &hunk.lines;
        while i < lines.len() {
            match lines[i].origin {
                DiffLineOrigin::Context => {
                    let line = &lines[i];
                    let gutter_l = format_lineno(line.old_lineno, gutter_width);
                    let gutter_r = format_lineno(line.new_lineno, gutter_width);
                    let old_spans = line.old_lineno.and_then(|n| old_hl.get(n as usize));
                    let new_spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                    left.push(make_highlighted_line(
                        &gutter_l,
                        &line.content,
                        old_spans,
                        None,
                    ));
                    right.push(make_highlighted_line(
                        &gutter_r,
                        &line.content,
                        new_spans,
                        None,
                    ));
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
                        if j < dels.len() {
                            let line = &dels[j];
                            let gutter = format_lineno(line.old_lineno, gutter_width);
                            let spans = line.old_lineno.and_then(|n| old_hl.get(n as usize));
                            left.push(make_highlighted_line(
                                &gutter,
                                &line.content,
                                spans,
                                Some(Color::Rgb(40, 0, 0)),
                            ));
                        } else {
                            left.push(make_empty_line(gutter_width));
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
                            ));
                        } else {
                            right.push(make_empty_line(gutter_width));
                        }
                    }
                }
                DiffLineOrigin::Addition => {
                    let line = &lines[i];
                    let gutter = format_lineno(line.new_lineno, gutter_width);
                    let spans = line.new_lineno.and_then(|n| new_hl.get(n as usize));
                    left.push(make_empty_line(gutter_width));
                    right.push(make_highlighted_line(
                        &gutter,
                        &line.content,
                        spans,
                        Some(Color::Rgb(0, 30, 0)),
                    ));
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

    let gutter_width = 5;
    let mut lines: Vec<Line> = Vec::new();

    for hunk in &delta.hunks {
        lines.push(Line::from(Span::styled(
            format!("{:>gutter_width$} {}", "...", &hunk.header),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));

        for line in &hunk.lines {
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
                    ));
                }
            }
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

/// Build a line with syntax highlighting spans overlaid on a diff background.
fn make_highlighted_line<'a>(
    gutter: &str,
    content: &str,
    hl_spans: Option<&Vec<HighlightSpan>>,
    bg: Option<Color>,
) -> Line<'a> {
    let trimmed = content.trim_end_matches('\n');
    let gutter_span = Span::styled(format!("{gutter} "), Style::default().fg(Color::DarkGray));

    let content_spans = if let Some(spans) = hl_spans {
        apply_highlights(trimmed, spans, bg)
    } else {
        // Fallback: no highlighting
        let mut style = Style::default();
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
            // Use diff-specific fg colors
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

fn make_empty_line<'a>(gutter_width: usize) -> Line<'a> {
    Line::from(Span::styled(
        format!("{} ", " ".repeat(gutter_width)),
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(20, 20, 20)),
    ))
}

fn make_unified_highlighted<'a>(
    old_g: &str,
    new_g: &str,
    prefix: &str,
    content: &str,
    hl_spans: Option<&Vec<HighlightSpan>>,
    bg: Option<Color>,
) -> Line<'a> {
    let trimmed = content.trim_end_matches('\n');
    let gutter_span = Span::styled(
        format!("{old_g} {new_g} "),
        Style::default().fg(Color::DarkGray),
    );

    let prefix_style = match prefix {
        "+" => Style::default().fg(Color::Green).bg(bg.unwrap_or_default()),
        "-" => Style::default().fg(Color::Red).bg(bg.unwrap_or_default()),
        _ => Style::default().fg(Color::DarkGray),
    };
    let prefix_span = Span::styled(prefix.to_string(), prefix_style);

    let content_spans = if let Some(spans) = hl_spans {
        apply_highlights(trimmed, spans, bg)
    } else {
        let mut style = Style::default().fg(Color::White);
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
        }
        vec![Span::styled(trimmed.to_string(), style)]
    };

    let mut all_spans = vec![gutter_span, prefix_span];
    all_spans.extend(content_spans);
    Line::from(all_spans)
}
