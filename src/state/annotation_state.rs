use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A file path + line range that anchors an annotation to specific diff lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAnchor {
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
}

/// A single annotation attached to a range of diff lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub anchor: LineAnchor,
    pub comment: String,
    pub created_at: String,
}

/// State for all annotations in the current session.
/// Keyed by file path for efficient lookup.
#[derive(Debug, Default)]
pub struct AnnotationState {
    /// Map of file_path → list of annotations on that file.
    pub annotations: BTreeMap<String, Vec<Annotation>>,
}

impl AnnotationState {
    /// Add an annotation for a file.
    pub fn add(&mut self, annotation: Annotation) {
        let key = annotation.anchor.file_path.clone();
        self.annotations.entry(key).or_default().push(annotation);
    }

    /// Check if any annotation covers the given line number in a file.
    pub fn has_annotation_at(&self, file_path: &str, lineno: u32) -> bool {
        if let Some(anns) = self.annotations.get(file_path) {
            anns.iter()
                .any(|a| lineno >= a.anchor.line_start && lineno <= a.anchor.line_end)
        } else {
            false
        }
    }

    /// Delete all annotations overlapping the given line range in a file.
    pub fn delete_at(&mut self, file_path: &str, line_start: u32, line_end: u32) {
        if let Some(anns) = self.annotations.get_mut(file_path) {
            anns.retain(|a| a.anchor.line_end < line_start || a.anchor.line_start > line_end);
            if anns.is_empty() {
                self.annotations.remove(file_path);
            }
        }
    }

    /// Get all annotations as a flat, sorted list (by file then line).
    pub fn all_sorted(&self) -> Vec<&Annotation> {
        let mut result: Vec<&Annotation> =
            self.annotations.values().flat_map(|v| v.iter()).collect();
        result.sort_by_key(|a| (&a.anchor.file_path, a.anchor.line_start));
        result
    }

    /// Return all annotations whose range overlaps the given line number.
    pub fn annotations_overlapping(&self, file_path: &str, lineno: u32) -> Vec<&Annotation> {
        if let Some(anns) = self.annotations.get(file_path) {
            anns.iter()
                .filter(|a| lineno >= a.anchor.line_start && lineno <= a.anchor.line_end)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Delete a specific annotation matching by anchor + comment text.
    pub fn delete_annotation(
        &mut self,
        file_path: &str,
        line_start: u32,
        line_end: u32,
        comment: &str,
    ) {
        if let Some(anns) = self.annotations.get_mut(file_path) {
            if let Some(pos) = anns.iter().position(|a| {
                a.anchor.line_start == line_start
                    && a.anchor.line_end == line_end
                    && a.comment == comment
            }) {
                anns.remove(pos);
            }
            if anns.is_empty() {
                self.annotations.remove(file_path);
            }
        }
    }

    /// Update a specific annotation's comment text.
    pub fn update_comment(
        &mut self,
        file_path: &str,
        line_start: u32,
        line_end: u32,
        old_comment: &str,
        new_comment: &str,
    ) {
        if let Some(anns) = self.annotations.get_mut(file_path) {
            if let Some(ann) = anns.iter_mut().find(|a| {
                a.anchor.line_start == line_start
                    && a.anchor.line_end == line_end
                    && a.comment == old_comment
            }) {
                ann.comment = new_comment.to_string();
            }
        }
    }

    /// Total count of annotations.
    pub fn count(&self) -> usize {
        self.annotations.values().map(|v| v.len()).sum()
    }

    /// Find the next annotation after the given file/line position.
    /// Returns (file_path, line_start) of the next annotation.
    pub fn next_after(&self, file_path: &str, lineno: u32) -> Option<(&str, u32)> {
        let sorted = self.all_sorted();
        // Find first annotation that comes after current position
        for ann in &sorted {
            if ann.anchor.file_path.as_str() > file_path
                || (ann.anchor.file_path == file_path && ann.anchor.line_start > lineno)
            {
                return Some((&ann.anchor.file_path, ann.anchor.line_start));
            }
        }
        // Wrap around to first
        sorted
            .first()
            .map(|a| (a.anchor.file_path.as_str(), a.anchor.line_start))
    }

    /// Find the previous annotation before the given file/line position.
    pub fn prev_before(&self, file_path: &str, lineno: u32) -> Option<(&str, u32)> {
        let sorted = self.all_sorted();
        // Find last annotation that comes before current position
        for ann in sorted.iter().rev() {
            if ann.anchor.file_path.as_str() < file_path
                || (ann.anchor.file_path == file_path && ann.anchor.line_start < lineno)
            {
                return Some((&ann.anchor.file_path, ann.anchor.line_start));
            }
        }
        // Wrap around to last
        sorted
            .last()
            .map(|a| (a.anchor.file_path.as_str(), a.anchor.line_start))
    }
}
