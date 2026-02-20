use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A file path + line ranges that anchor an annotation to specific diff lines.
/// Stores old-file and new-file ranges separately so the LLM prompt can
/// distinguish between deleted, added, and context lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAnchor {
    pub file_path: String,
    pub old_range: Option<(u32, u32)>, // (start, end) in old file
    pub new_range: Option<(u32, u32)>, // (start, end) in new file
}

impl LineAnchor {
    /// A single representative line number for sorting/navigation.
    /// Prefers new-file start, falls back to old-file start.
    pub fn sort_line(&self) -> u32 {
        self.new_range
            .map(|(s, _)| s)
            .or(self.old_range.map(|(s, _)| s))
            .unwrap_or(0)
    }

    /// Whether this anchor covers a given old-file line number.
    pub fn covers_old(&self, lineno: u32) -> bool {
        self.old_range
            .is_some_and(|(s, e)| lineno >= s && lineno <= e)
    }

    /// Whether this anchor covers a given new-file line number.
    pub fn covers_new(&self, lineno: u32) -> bool {
        self.new_range
            .is_some_and(|(s, e)| lineno >= s && lineno <= e)
    }

    /// Whether this anchor covers the given old/new line number pair.
    /// Returns true if either side matches (when present).
    pub fn covers(&self, old_lineno: Option<u32>, new_lineno: Option<u32>) -> bool {
        if let Some(n) = new_lineno {
            if self.covers_new(n) {
                return true;
            }
        }
        if let Some(n) = old_lineno {
            if self.covers_old(n) {
                return true;
            }
        }
        false
    }

    /// Check if this anchor's ranges overlap with the given ranges.
    fn overlaps(&self, old_range: Option<(u32, u32)>, new_range: Option<(u32, u32)>) -> bool {
        let old_overlaps = match (self.old_range, old_range) {
            (Some((s1, e1)), Some((s2, e2))) => s1 <= e2 && s2 <= e1,
            _ => false,
        };
        let new_overlaps = match (self.new_range, new_range) {
            (Some((s1, e1)), Some((s2, e2))) => s1 <= e2 && s2 <= e1,
            _ => false,
        };
        old_overlaps || new_overlaps
    }

    /// Check if this anchor exactly matches the given ranges.
    fn matches(&self, old_range: Option<(u32, u32)>, new_range: Option<(u32, u32)>) -> bool {
        self.old_range == old_range && self.new_range == new_range
    }
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

    /// Check if any annotation covers the given line numbers in a file.
    pub fn has_annotation_at(
        &self,
        file_path: &str,
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
    ) -> bool {
        if let Some(anns) = self.annotations.get(file_path) {
            anns.iter().any(|a| a.anchor.covers(old_lineno, new_lineno))
        } else {
            false
        }
    }

    /// Delete all annotations overlapping the given ranges in a file.
    pub fn delete_at(
        &mut self,
        file_path: &str,
        old_range: Option<(u32, u32)>,
        new_range: Option<(u32, u32)>,
    ) {
        if let Some(anns) = self.annotations.get_mut(file_path) {
            anns.retain(|a| !a.anchor.overlaps(old_range, new_range));
            if anns.is_empty() {
                self.annotations.remove(file_path);
            }
        }
    }

    /// Get all annotations as a flat, sorted list (by file then sort_line).
    pub fn all_sorted(&self) -> Vec<&Annotation> {
        let mut result: Vec<&Annotation> =
            self.annotations.values().flat_map(|v| v.iter()).collect();
        result.sort_by_key(|a| (&a.anchor.file_path, a.anchor.sort_line()));
        result
    }

    /// Return all annotations whose range covers the given line numbers.
    pub fn annotations_overlapping(
        &self,
        file_path: &str,
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
    ) -> Vec<&Annotation> {
        if let Some(anns) = self.annotations.get(file_path) {
            anns.iter()
                .filter(|a| a.anchor.covers(old_lineno, new_lineno))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Delete a specific annotation matching by anchor ranges + comment text.
    pub fn delete_annotation(
        &mut self,
        file_path: &str,
        old_range: Option<(u32, u32)>,
        new_range: Option<(u32, u32)>,
        comment: &str,
    ) {
        if let Some(anns) = self.annotations.get_mut(file_path) {
            if let Some(pos) = anns
                .iter()
                .position(|a| a.anchor.matches(old_range, new_range) && a.comment == comment)
            {
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
        old_range: Option<(u32, u32)>,
        new_range: Option<(u32, u32)>,
        old_comment: &str,
        new_comment: &str,
    ) {
        if let Some(anns) = self.annotations.get_mut(file_path) {
            if let Some(ann) = anns
                .iter_mut()
                .find(|a| a.anchor.matches(old_range, new_range) && a.comment == old_comment)
            {
                ann.comment = new_comment.to_string();
            }
        }
    }

    /// Total count of annotations.
    pub fn count(&self) -> usize {
        self.annotations.values().map(|v| v.len()).sum()
    }

    /// Find the next annotation after the given file/line position.
    /// Returns (file_path, sort_line) of the next annotation.
    pub fn next_after(&self, file_path: &str, lineno: u32) -> Option<(&str, u32)> {
        let sorted = self.all_sorted();
        for ann in &sorted {
            let sl = ann.anchor.sort_line();
            if ann.anchor.file_path.as_str() > file_path
                || (ann.anchor.file_path == file_path && sl > lineno)
            {
                return Some((&ann.anchor.file_path, sl));
            }
        }
        // Wrap around to first
        sorted
            .first()
            .map(|a| (a.anchor.file_path.as_str(), a.anchor.sort_line()))
    }

    /// Find the previous annotation before the given file/line position.
    pub fn prev_before(&self, file_path: &str, lineno: u32) -> Option<(&str, u32)> {
        let sorted = self.all_sorted();
        for ann in sorted.iter().rev() {
            let sl = ann.anchor.sort_line();
            if ann.anchor.file_path.as_str() < file_path
                || (ann.anchor.file_path == file_path && sl < lineno)
            {
                return Some((&ann.anchor.file_path, sl));
            }
        }
        // Wrap around to last
        sorted
            .last()
            .map(|a| (a.anchor.file_path.as_str(), a.anchor.sort_line()))
    }
}
