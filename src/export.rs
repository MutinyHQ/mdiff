use chrono::Utc;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::state::review_state::FileReviewStatus;
use crate::state::AppState;

#[derive(Serialize)]
pub struct FeedbackExport {
    pub version: u32,
    pub exported_at: String,
    pub target: String,
    pub summary: FeedbackSummary,
    pub files: Vec<FileFeedback>,
    pub decision: Option<String>,
}

#[derive(Serialize)]
pub struct FeedbackSummary {
    pub total_files: usize,
    pub files_reviewed: usize,
    pub files_with_annotations: usize,
    pub total_annotations: usize,
    pub total_line_scores: usize,
    pub review_completeness: f64,
}

#[derive(Serialize)]
pub struct FileFeedback {
    pub path: String,
    pub review_status: String,
    pub additions: usize,
    pub deletions: usize,
    pub annotations: Vec<AnnotationExport>,
    pub line_scores: Vec<LineScoreExport>,
}

#[derive(Serialize)]
pub struct AnnotationExport {
    #[serde(rename = "type")]
    pub annotation_type: String,
    pub old_range: Option<(u32, u32)>,
    pub new_range: Option<(u32, u32)>,
    pub comment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

#[derive(Serialize)]
pub struct LineScoreExport {
    pub line: u32,
    pub score: u8,
    pub side: String,
}

pub fn export_feedback(
    state: &AppState,
    repo_path: &Path,
    target_label: &str,
) -> Result<PathBuf, String> {
    let export = build_export(state, target_label);

    let dir = repo_path.join(".mdiff-feedback");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!(
        "feedback_{}_{}.json",
        target_label.replace(['/', '\\', ':', ' '], "_"),
        timestamp
    );
    let path = dir.join(&filename);

    let json =
        serde_json::to_string_pretty(&export).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(path)
}

fn build_export(state: &AppState, target_label: &str) -> FeedbackExport {
    let total_files = state.diff.deltas.len();
    let files_reviewed = count_reviewed_files(state);
    let files_with_annotations = state.annotations.files_with_annotations();
    let total_annotations = state.annotations.count();
    let total_line_scores = state.annotations.score_count();

    let review_completeness = if total_files > 0 {
        files_reviewed as f64 / total_files as f64
    } else {
        0.0
    };

    let summary = FeedbackSummary {
        total_files,
        files_reviewed,
        files_with_annotations,
        total_annotations,
        total_line_scores,
        review_completeness,
    };

    let mut files = Vec::new();

    for delta in state.diff.deltas.iter() {
        let path = delta.path.to_string_lossy().to_string();
        let review_status = match state.review.status(&path) {
            FileReviewStatus::Unreviewed => "unreviewed".to_string(),
            FileReviewStatus::Reviewed => "reviewed".to_string(),
            FileReviewStatus::ChangedSinceReview => "changed_since_review".to_string(),
            FileReviewStatus::New => "new".to_string(),
        };

        // Count additions and deletions
        let mut additions = 0;
        let mut deletions = 0;
        for hunk in &delta.hunks {
            for line in &hunk.lines {
                match line.origin {
                    crate::git::types::DiffLineOrigin::Addition => additions += 1,
                    crate::git::types::DiffLineOrigin::Deletion => deletions += 1,
                    crate::git::types::DiffLineOrigin::Context => {}
                }
            }
        }

        // Get annotations for this file
        let annotations = if let Some(file_annotations) = state.annotations.annotations.get(&path) {
            file_annotations
                .iter()
                .map(|ann| AnnotationExport {
                    annotation_type: "comment".to_string(),
                    old_range: ann.anchor.old_range,
                    new_range: ann.anchor.new_range,
                    comment: ann.comment.clone(),
                    category: None, // Could be extended in the future
                    severity: None, // Could be extended in the future
                })
                .collect()
        } else {
            Vec::new()
        };

        // Get line scores for this file (placeholder - not implemented in current system)
        let line_scores = Vec::new();

        files.push(FileFeedback {
            path,
            review_status,
            additions,
            deletions,
            annotations,
            line_scores,
        });
    }

    FeedbackExport {
        version: 1,
        exported_at: Utc::now().to_rfc3339(),
        target: target_label.to_string(),
        summary,
        files,
        decision: None, // Could be extended to include overall review decision
    }
}

fn count_reviewed_files(state: &AppState) -> usize {
    state
        .diff
        .deltas
        .iter()
        .filter(|delta| {
            let path = delta.path.to_string_lossy().to_string();
            matches!(state.review.status(&path), FileReviewStatus::Reviewed)
        })
        .count()
}

/// Ensure `.mdiff-feedback/` is listed in `.gitignore`.
pub fn ensure_gitignore(repo_path: &Path) {
    let gitignore_path = repo_path.join(".gitignore");
    let entry = ".mdiff-feedback/";

    if let Ok(contents) = std::fs::read_to_string(&gitignore_path) {
        if contents.lines().any(|line| line.trim() == entry) {
            return;
        }
        // Append to existing .gitignore
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open(&gitignore_path)
        {
            use std::io::Write;
            // Add newline if file doesn't end with one
            if !contents.ends_with('\n') {
                let _ = writeln!(f);
            }
            let _ = writeln!(f, "{entry}");
        }
    } else {
        // No .gitignore yet — create one
        let _ = std::fs::write(&gitignore_path, format!("{entry}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, DiffOptions};
    use crate::theme::Theme;
    use std::path::Path;

    #[test]
    fn test_export_feedback_creates_valid_json() {
        // Create a test app state
        let diff_options = DiffOptions::new(false, false);
        let theme = Theme::from_name("one-dark");
        let mut state = AppState::new(diff_options, theme);
        state.target_label = "HEAD".to_string();
        
        // Test the export functionality
        let repo_path = Path::new(".");
        
        // Test export
        let result = export_feedback(&state, &repo_path, &state.target_label);
        assert!(result.is_ok(), "Export should succeed");
        
        let path = result.unwrap();
        
        // Verify the file exists and contains valid JSON
        let content = std::fs::read_to_string(&path).expect("Should be able to read export file");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("Should be valid JSON");
        
        // Verify basic structure
        assert!(parsed["version"].is_number());
        assert!(parsed["exported_at"].is_string());
        assert!(parsed["target"].is_string());
        assert!(parsed["summary"].is_object());
        assert!(parsed["files"].is_array());
        
        // Clean up test file
        std::fs::remove_file(&path).expect("Should be able to clean up test file");
    }

    #[test]
    fn test_build_export_structure() {
        let diff_options = DiffOptions::new(false, false);
        let theme = Theme::from_name("one-dark");
        let state = AppState::new(diff_options, theme);
        
        let export = build_export(&state, "test-target");
        
        assert_eq!(export.version, 1);
        assert_eq!(export.target, "test-target");
        assert!(export.exported_at.len() > 0);
        assert_eq!(export.summary.total_files, 0); // No deltas in empty state
        assert!(export.files.is_empty());
    }
}
