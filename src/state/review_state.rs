use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::git::types::FileDelta;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileReviewStatus {
    /// Never viewed.
    Unreviewed,
    /// Viewed and diff content unchanged since last review.
    Reviewed,
    /// Viewed previously but diff content changed on refresh.
    ChangedSinceReview,
    /// File appeared after a diff refresh (not present before).
    New,
}

/// Tracks per-file review progress. In-memory only (resets on quit).
#[derive(Debug, Default)]
pub struct ReviewState {
    /// Status and the hash at time of review (if reviewed).
    files: HashMap<String, (FileReviewStatus, Option<u64>)>,
    /// Hashes from the most recent diff load.
    current_hashes: HashMap<String, u64>,
}

impl ReviewState {
    /// Mark a file as reviewed, storing the current diff hash.
    pub fn mark_reviewed(&mut self, path: &str) {
        let hash = self.current_hashes.get(path).copied();
        self.files
            .insert(path.to_string(), (FileReviewStatus::Reviewed, hash));
    }

    /// Toggle between Reviewed and Unreviewed.
    pub fn toggle_reviewed(&mut self, path: &str) {
        match self.files.get(path).map(|(s, _)| *s) {
            Some(FileReviewStatus::Reviewed) => {
                self.files
                    .insert(path.to_string(), (FileReviewStatus::Unreviewed, None));
            }
            _ => {
                self.mark_reviewed(path);
            }
        }
    }

    /// Get the review status for a file.
    pub fn status(&self, path: &str) -> FileReviewStatus {
        self.files
            .get(path)
            .map(|(s, _)| *s)
            .unwrap_or(FileReviewStatus::Unreviewed)
    }

    /// Called after a diff refresh with new hashes. Compares against stored
    /// reviewed hashes to detect changes.
    pub fn on_diff_refresh(&mut self, new_hashes: HashMap<String, u64>) {
        let first_load = self.current_hashes.is_empty();

        if first_load {
            // First diff load: mark all files as Unreviewed (not New).
            for path in new_hashes.keys() {
                self.files
                    .entry(path.clone())
                    .or_insert((FileReviewStatus::Unreviewed, None));
            }
        } else {
            let old_paths: std::collections::HashSet<&String> =
                self.current_hashes.keys().collect();

            for (path, &new_hash) in &new_hashes {
                if !old_paths.contains(path) {
                    // File is new since last refresh.
                    self.files
                        .insert(path.clone(), (FileReviewStatus::New, None));
                } else if let Some((
                    FileReviewStatus::Reviewed | FileReviewStatus::ChangedSinceReview,
                    reviewed_hash,
                )) = self.files.get(path)
                {
                    // Check if diff content changed since review.
                    if *reviewed_hash != Some(new_hash) {
                        self.files.insert(
                            path.clone(),
                            (FileReviewStatus::ChangedSinceReview, *reviewed_hash),
                        );
                    } else {
                        // Hash matches review - stays Reviewed.
                        self.files
                            .insert(path.clone(), (FileReviewStatus::Reviewed, *reviewed_hash));
                    }
                }
            }

            // Files that disappeared: remove from tracking.
            let new_paths: std::collections::HashSet<&String> = new_hashes.keys().collect();
            self.files.retain(|k, _| new_paths.contains(k));
        }

        self.current_hashes = new_hashes;
    }

    /// Reset all review state (e.g. on target/worktree change).
    pub fn reset(&mut self) {
        self.files.clear();
        self.current_hashes.clear();
    }
}

/// Compute a hash fingerprint of a FileDelta's diff content.
pub fn hash_file_diff(delta: &FileDelta) -> u64 {
    let mut hasher = DefaultHasher::new();
    for hunk in &delta.hunks {
        hunk.header.hash(&mut hasher);
        for line in &hunk.lines {
            line.content.hash(&mut hasher);
            match line.origin {
                crate::git::types::DiffLineOrigin::Context => 0u8.hash(&mut hasher),
                crate::git::types::DiffLineOrigin::Addition => 1u8.hash(&mut hasher),
                crate::git::types::DiffLineOrigin::Deletion => 2u8.hash(&mut hasher),
            }
        }
    }
    hasher.finish()
}

/// Compute hashes for all deltas, keyed by file path.
pub fn compute_diff_hashes(deltas: &[FileDelta]) -> HashMap<String, u64> {
    deltas
        .iter()
        .map(|d| {
            let path = d.path.to_string_lossy().to_string();
            let hash = hash_file_diff(d);
            (path, hash)
        })
        .collect()
}
