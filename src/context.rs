use crate::git::types::{DiffLineOrigin, FileDelta};
use crate::state::annotation_state::LineAnchor;
use crate::template::TemplateContext;

/// Extract context from a diff delta for template rendering.
///
/// Walks the delta's hunks to find lines overlapping the anchor range,
/// collects diff lines with +/- prefixes, and gathers surrounding padding lines.
pub fn extract_context(
    delta: &FileDelta,
    anchor: &LineAnchor,
    comment: &str,
    padding: usize,
) -> TemplateContext {
    let mut diff_lines: Vec<String> = Vec::new();
    let mut context_lines: Vec<String> = Vec::new();
    let mut hunk_header = String::new();

    let padded_start = anchor.line_start.saturating_sub(padding as u32);
    let padded_end = anchor.line_end + padding as u32;

    for hunk in &delta.hunks {
        let mut found_in_hunk = false;

        for line in &hunk.lines {
            // Check if this line falls within the anchor range (using either line number)
            let lineno = line.new_lineno.or(line.old_lineno).unwrap_or(0);
            let in_range = lineno >= anchor.line_start && lineno <= anchor.line_end;
            let in_padded = lineno >= padded_start && lineno <= padded_end;

            if in_range {
                found_in_hunk = true;
                let prefix = match line.origin {
                    DiffLineOrigin::Addition => "+",
                    DiffLineOrigin::Deletion => "-",
                    DiffLineOrigin::Context => " ",
                };
                diff_lines.push(format!("{}{}", prefix, line.content.trim_end()));
            }

            if in_padded && !in_range {
                let prefix = match line.origin {
                    DiffLineOrigin::Addition => "+",
                    DiffLineOrigin::Deletion => "-",
                    DiffLineOrigin::Context => " ",
                };
                context_lines.push(format!("{}{}", prefix, line.content.trim_end()));
            }
        }

        if found_in_hunk && hunk_header.is_empty() {
            hunk_header = hunk.header.clone();
        }
    }

    TemplateContext {
        filename: anchor.file_path.clone(),
        line_start: anchor.line_start,
        line_end: anchor.line_end,
        diff_content: diff_lines.join("\n"),
        context: context_lines.join("\n"),
        comments: comment.to_string(),
        hunk_header,
    }
}
