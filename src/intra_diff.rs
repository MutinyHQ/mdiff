use std::collections::HashMap;
use similar::{ChangeTag, TextDiff};
use crate::git::types::{DiffLine, DiffLineOrigin};

/// Intra-line change spans for a single diff line.
#[derive(Debug, Clone)]
pub struct IntraLineSpan {
    pub start: usize,  // byte offset in content string
    pub end: usize,    // byte offset (exclusive)
}

/// Convert character index to byte index in a string.
fn char_to_byte_index(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(s.len())
}

/// Compute intra-line change spans for a pair of old/new lines.
pub fn compute_intra_line_spans(
    old_line: &str,
    new_line: &str,
) -> (Vec<IntraLineSpan>, Vec<IntraLineSpan>) {
    // Cap computation at 500 chars per line for performance
    if old_line.len() > 500 || new_line.len() > 500 {
        return (Vec::new(), Vec::new());
    }

    let diff = TextDiff::from_chars(old_line, new_line);
    
    let mut old_spans = Vec::new();
    let mut new_spans = Vec::new();
    let mut old_char_offset = 0usize;
    let mut new_char_offset = 0usize;
    
    for change in diff.iter_all_changes() {
        let char_len = change.value().chars().count();
        match change.tag() {
            ChangeTag::Equal => {
                old_char_offset += char_len;
                new_char_offset += char_len;
            }
            ChangeTag::Delete => {
                let start_byte = char_to_byte_index(old_line, old_char_offset);
                let end_byte = char_to_byte_index(old_line, old_char_offset + char_len);
                old_spans.push(IntraLineSpan {
                    start: start_byte,
                    end: end_byte,
                });
                old_char_offset += char_len;
            }
            ChangeTag::Insert => {
                let start_byte = char_to_byte_index(new_line, new_char_offset);
                let end_byte = char_to_byte_index(new_line, new_char_offset + char_len);
                new_spans.push(IntraLineSpan {
                    start: start_byte,
                    end: end_byte,
                });
                new_char_offset += char_len;
            }
        }
    }
    
    // Merge adjacent spans
    let old_spans = merge_adjacent_spans(old_spans);
    let new_spans = merge_adjacent_spans(new_spans);
    
    // If most of the line is changed (>80%), don't highlight
    // The whole-line coloring is sufficient in this case
    let old_changed_chars: usize = old_spans.iter().map(|s| s.end - s.start).sum();
    let new_changed_chars: usize = new_spans.iter().map(|s| s.end - s.start).sum();
    
    let old_total_chars = old_line.len();
    let new_total_chars = new_line.len();
    
    if (old_total_chars > 0 && old_changed_chars * 100 / old_total_chars > 80) ||
       (new_total_chars > 0 && new_changed_chars * 100 / new_total_chars > 80) {
        return (Vec::new(), Vec::new());
    }
    
    (old_spans, new_spans)
}

/// Merge adjacent and nearby IntraLineSpans into larger spans.
fn merge_adjacent_spans(spans: Vec<IntraLineSpan>) -> Vec<IntraLineSpan> {
    if spans.is_empty() {
        return spans;
    }
    
    let mut merged = Vec::new();
    let mut current = spans[0].clone();
    
    for span in spans.into_iter().skip(1) {
        // Merge if adjacent or if there's only a small gap (1-2 characters)
        if span.start <= current.end + 2 {
            current.end = span.end;
        } else {
            // Gap is too large, push current and start new one
            merged.push(current);
            current = span;
        }
    }
    
    merged.push(current);
    merged
}

/// A paired change: old lines removed, new lines added.
#[derive(Debug)]
struct ChangePair {
    old_lines: Vec<(usize, String)>,  // (hunk_line_index, content)
    new_lines: Vec<(usize, String)>,  // (hunk_line_index, content)
}

/// Scan a hunk's lines and compute intra-line highlights for all change pairs.
pub fn compute_hunk_intra_highlights(
    lines: &[DiffLine],
) -> HashMap<usize, Vec<IntraLineSpan>> {
    let mut highlights = HashMap::new();
    let mut change_groups = Vec::new();
    
    // 1. Identify change groups (consecutive deletions followed by additions)
    let mut i = 0;
    while i < lines.len() {
        if lines[i].origin == DiffLineOrigin::Deletion {
            let mut old_lines = Vec::new();
            
            // Collect consecutive deletions
            while i < lines.len() && lines[i].origin == DiffLineOrigin::Deletion {
                old_lines.push((i, lines[i].content.clone()));
                i += 1;
            }
            
            // Check if followed by consecutive additions
            let mut new_lines = Vec::new();
            let _start_additions = i;
            while i < lines.len() && lines[i].origin == DiffLineOrigin::Addition {
                new_lines.push((i, lines[i].content.clone()));
                i += 1;
            }
            
            // If we have both deletions and additions, it's a change group
            if !old_lines.is_empty() && !new_lines.is_empty() {
                change_groups.push(ChangePair { old_lines, new_lines });
            }
            
            // If we only had deletions without additions, continue from where we are
            // If we had both, we already advanced past the additions
        } else {
            i += 1;
        }
    }
    
    // 2. For equal-count groups, pair 1:1 and compute spans
    for group in change_groups {
        if group.old_lines.len() == group.new_lines.len() {
            // Pair lines 1:1
            for (old_entry, new_entry) in group.old_lines.iter().zip(group.new_lines.iter()) {
                let (old_spans, new_spans) = compute_intra_line_spans(&old_entry.1, &new_entry.1);
                
                if !old_spans.is_empty() {
                    highlights.insert(old_entry.0, old_spans);
                }
                if !new_spans.is_empty() {
                    highlights.insert(new_entry.0, new_spans);
                }
            }
        }
        // For unequal counts, fall back to no intra-line highlighting
        // (likely a structural change, not a modification)
    }
    
    highlights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_character_change() {
        let old = "hello world";
        let new = "hello earth";
        let (old_spans, new_spans) = compute_intra_line_spans(old, new);
        
        
        // Should highlight "world" -> "earth"
        assert_eq!(old_spans.len(), 1);
        assert_eq!(old_spans[0].start, 6);
        assert_eq!(old_spans[0].end, 11);
        
        assert_eq!(new_spans.len(), 1);
        assert_eq!(new_spans[0].start, 6);
        assert_eq!(new_spans[0].end, 11);
    }

    #[test]
    fn test_word_addition() {
        let old = "hello world";
        let new = "hello beautiful world";
        let (old_spans, new_spans) = compute_intra_line_spans(old, new);
        
        // Should highlight the added "beautiful "
        assert_eq!(old_spans.len(), 0); // Nothing removed
        assert_eq!(new_spans.len(), 1);
        assert_eq!(new_spans[0].start, 6);
        assert_eq!(new_spans[0].end, 16); // "beautiful "
    }

    #[test]
    fn test_entire_line_changed() {
        let old = "abcdefghijklmnop";
        let new = "qrstuvwxyz123456";
        let (old_spans, new_spans) = compute_intra_line_spans(old, new);
        
        // Should return empty spans when entire line is changed (>80% different)
        assert_eq!(old_spans.len(), 0);
        assert_eq!(new_spans.len(), 0);
    }

    #[test]
    fn test_very_long_line() {
        let old = "a".repeat(600);
        let new = "b".repeat(600);
        let (old_spans, new_spans) = compute_intra_line_spans(&old, &new);
        
        // Should return empty spans for lines > 500 chars
        assert_eq!(old_spans.len(), 0);
        assert_eq!(new_spans.len(), 0);
    }

    #[test]
    fn test_unicode_handling() {
        let old = "hello 世界";
        let new = "hello 🌍";
        let (old_spans, new_spans) = compute_intra_line_spans(old, new);
        
        // Should handle unicode correctly
        assert_eq!(old_spans.len(), 1);
        assert_eq!(new_spans.len(), 1);
        // The exact byte offsets depend on UTF-8 encoding
        assert!(old_spans[0].start >= 6);
        assert!(new_spans[0].start >= 6);
    }
}