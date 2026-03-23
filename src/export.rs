use chrono::Utc;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::state::annotation_state::{AnnotationCategory, AnnotationSeverity};
use crate::state::review_state::FileReviewStatus;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Markdown,
    AgentGuidance,
    ClaudeMd,
    GitTrailers,
}

impl ExportFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Markdown => "Markdown",
            Self::AgentGuidance => "Agent Guidance",
            Self::ClaudeMd => "CLAUDE.md Patch",
            Self::GitTrailers => "Git Trailers",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            Self::Json => 'J',
            Self::Markdown => 'M',
            Self::AgentGuidance => 'A',
            Self::ClaudeMd => 'C',
            Self::GitTrailers => 'G',
        }
    }

    pub fn all() -> &'static [ExportFormat] {
        &[
            Self::Json,
            Self::Markdown,
            Self::AgentGuidance,
            Self::ClaudeMd,
            Self::GitTrailers,
        ]
    }
}

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
                    category: Some(ann.category.label().to_string()),
                    severity: Some(ann.severity.label().to_string()),
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

/// Export feedback in the specified format. Returns the file path for file-based
/// formats, or the generated text for clipboard-based formats.
pub fn export_with_format(
    state: &AppState,
    repo_path: &Path,
    target_label: &str,
    format: ExportFormat,
) -> Result<ExportResult, String> {
    match format {
        ExportFormat::Json => {
            let path = export_feedback(state, repo_path, target_label)?;
            Ok(ExportResult::File(path))
        }
        ExportFormat::Markdown => {
            let text = generate_markdown(state, target_label);
            let path = write_export_file(repo_path, target_label, &text, "md")?;
            Ok(ExportResult::File(path))
        }
        ExportFormat::AgentGuidance => {
            let text = generate_agent_guidance(state, target_label);
            Ok(ExportResult::Text(text))
        }
        ExportFormat::ClaudeMd => {
            let text = generate_claude_md_patch(state, target_label);
            Ok(ExportResult::Text(text))
        }
        ExportFormat::GitTrailers => {
            let text = generate_git_trailers(state, target_label);
            Ok(ExportResult::Text(text))
        }
    }
}

pub enum ExportResult {
    File(PathBuf),
    Text(String),
}

fn write_export_file(
    repo_path: &Path,
    target_label: &str,
    content: &str,
    extension: &str,
) -> Result<PathBuf, String> {
    let dir = repo_path.join(".mdiff-feedback");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!(
        "feedback_{}_{}.{}",
        target_label.replace(['/', '\\', ':', ' '], "_"),
        timestamp,
        extension,
    );
    let path = dir.join(&filename);
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(path)
}

/// Structured annotation data used by the guidance generators.
struct GuidanceAnnotation {
    file_path: String,
    line_ref: String,
    comment: String,
    category: AnnotationCategory,
    severity: AnnotationSeverity,
}

fn collect_guidance_annotations(state: &AppState) -> Vec<GuidanceAnnotation> {
    let mut out = Vec::new();
    for ann in state.annotations.all_sorted() {
        let line_ref = match (ann.anchor.old_range, ann.anchor.new_range) {
            (_, Some((s, e))) if s == e => format!("{}:{}", ann.anchor.file_path, s),
            (_, Some((s, e))) => format!("{}:{}-{}", ann.anchor.file_path, s, e),
            (Some((s, e)), None) if s == e => {
                format!("{} (removed line {})", ann.anchor.file_path, s)
            }
            (Some((s, e)), None) => {
                format!("{} (removed lines {}-{})", ann.anchor.file_path, s, e)
            }
            (None, None) => ann.anchor.file_path.clone(),
        };
        out.push(GuidanceAnnotation {
            file_path: ann.anchor.file_path.clone(),
            line_ref,
            comment: ann.comment.clone(),
            category: ann.category,
            severity: ann.severity,
        });
    }
    out
}

/// Classify an annotation into a guidance section.
enum GuidanceSection {
    Critical,
    Suggestion,
    StyleNote,
    Question,
    Positive,
}

fn classify_annotation(ann: &GuidanceAnnotation, scores: &[(String, u8)]) -> GuidanceSection {
    let has_positive_score = scores.iter().any(|(fp, s)| fp == &ann.file_path && *s >= 4);

    if has_positive_score {
        return GuidanceSection::Positive;
    }

    match ann.category {
        AnnotationCategory::Question => GuidanceSection::Question,
        AnnotationCategory::Bug | AnnotationCategory::Security => match ann.severity {
            AnnotationSeverity::Critical | AnnotationSeverity::Major => GuidanceSection::Critical,
            _ => GuidanceSection::Suggestion,
        },
        AnnotationCategory::Performance => match ann.severity {
            AnnotationSeverity::Critical | AnnotationSeverity::Major => GuidanceSection::Critical,
            _ => GuidanceSection::Suggestion,
        },
        AnnotationCategory::Style | AnnotationCategory::Nitpick => GuidanceSection::StyleNote,
        AnnotationCategory::Suggestion => match ann.severity {
            AnnotationSeverity::Critical => GuidanceSection::Critical,
            _ => GuidanceSection::Suggestion,
        },
    }
}

fn collect_file_scores(state: &AppState) -> Vec<(String, u8)> {
    state
        .annotations
        .all_scores_sorted()
        .iter()
        .map(|s| (s.file_path.clone(), s.score))
        .collect()
}

/// Generate the Agent Instruction Format (direct prompt for agents).
pub fn generate_agent_guidance(state: &AppState, target_label: &str) -> String {
    let annotations = collect_guidance_annotations(state);
    if annotations.is_empty() {
        return "No annotations to export.".to_string();
    }

    let scores = collect_file_scores(state);

    let mut critical = Vec::new();
    let mut suggestions = Vec::new();
    let mut style_notes = Vec::new();
    let mut questions = Vec::new();
    let mut positive = Vec::new();

    for ann in &annotations {
        match classify_annotation(ann, &scores) {
            GuidanceSection::Critical => critical.push(ann),
            GuidanceSection::Suggestion => suggestions.push(ann),
            GuidanceSection::StyleNote => style_notes.push(ann),
            GuidanceSection::Question => questions.push(ann),
            GuidanceSection::Positive => positive.push(ann),
        }
    }

    let mut out = String::new();
    out.push_str(&format!(
        "I reviewed your changes in {}. Here is my structured feedback:\n",
        target_label
    ));

    let mut item_num = 1u32;

    if !critical.is_empty() {
        out.push_str("\n## Critical Issues (must fix)\n");
        for ann in &critical {
            let cat_label = ann.category.label().to_uppercase();
            out.push_str(&format!(
                "{}. [{}] {}: {}\n",
                item_num, ann.line_ref, cat_label, ann.comment
            ));
            item_num += 1;
        }
    }

    if !suggestions.is_empty() {
        out.push_str("\n## Suggestions (should fix)\n");
        for ann in &suggestions {
            let cat_label = ann.category.label().to_uppercase();
            out.push_str(&format!(
                "{}. [{}] {}: {}\n",
                item_num, ann.line_ref, cat_label, ann.comment
            ));
            item_num += 1;
        }
    }

    if !style_notes.is_empty() {
        out.push_str("\n## Style Notes (low priority)\n");
        for ann in &style_notes {
            out.push_str(&format!(
                "{}. [{}] {}\n",
                item_num, ann.line_ref, ann.comment
            ));
            item_num += 1;
        }
    }

    if !questions.is_empty() {
        out.push_str("\n## Questions for Clarification\n");
        for ann in &questions {
            out.push_str(&format!(
                "{}. [{}] {}\n",
                item_num, ann.line_ref, ann.comment
            ));
            item_num += 1;
        }
    }

    if !positive.is_empty() {
        out.push_str("\n## Positive Observations (keep doing)\n");
        for ann in &positive {
            out.push_str(&format!(
                "{}. [{}] {}\n",
                item_num, ann.line_ref, ann.comment
            ));
            item_num += 1;
        }
    }

    let last_item = item_num - 1;
    if !critical.is_empty() {
        let crit_max = critical.len() as u32;
        out.push_str(&format!(
            "\nPlease address items 1-{} as critical fixes",
            crit_max
        ));
        if !suggestions.is_empty() {
            out.push_str(&format!(
                ", then items {}-{} as improvements",
                crit_max + 1,
                crit_max + suggestions.len() as u32
            ));
        }
        out.push('.');
    }
    if !positive.is_empty() {
        out.push_str(&format!(
            " Do not change the patterns noted in item{}{}.\n",
            if positive.len() > 1 { "s " } else { " " },
            (last_item - positive.len() as u32 + 1..=last_item)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        out.push('\n');
    }

    let char_count = out.len();
    let approx_tokens = char_count / 4;
    if approx_tokens > 4000 {
        out.push_str(&format!(
            "\n⚠ Warning: This guidance is approximately {} tokens. Consider splitting into smaller chunks.\n",
            approx_tokens
        ));
    }

    out
}

/// Generate CLAUDE.md / AGENTS.md patch format.
pub fn generate_claude_md_patch(state: &AppState, target_label: &str) -> String {
    let annotations = collect_guidance_annotations(state);
    if annotations.is_empty() {
        return "No annotations to export.".to_string();
    }

    let scores = collect_file_scores(state);

    let mut avoid = Vec::new();
    let mut follow = Vec::new();
    let mut fixes = Vec::new();

    for ann in &annotations {
        match classify_annotation(ann, &scores) {
            GuidanceSection::Critical | GuidanceSection::Suggestion => {
                // Annotations with specific line ranges go to fixes, others to avoid
                if ann.line_ref.contains(':') {
                    fixes.push(ann);
                }
                avoid.push(ann);
            }
            GuidanceSection::Positive => follow.push(ann),
            GuidanceSection::StyleNote => avoid.push(ann),
            GuidanceSection::Question => {}
        }
    }

    let mut out = String::new();
    out.push_str(&format!(
        "## Review Feedback (from mdiff review of {})\n",
        target_label
    ));

    if !avoid.is_empty() {
        out.push_str("\n### Patterns to Avoid\n");
        for ann in &avoid {
            let sev_label = ann.severity.label().to_lowercase();
            let cat_label = ann.category.label().to_lowercase();
            out.push_str(&format!(
                "- In `{}`: {} (Severity: {}, Category: {})\n",
                ann.file_path, ann.comment, sev_label, cat_label
            ));
        }
    }

    if !follow.is_empty() {
        out.push_str("\n### Patterns to Follow\n");
        for ann in &follow {
            out.push_str(&format!("- In `{}`: {}\n", ann.file_path, ann.comment));
        }
    }

    if !fixes.is_empty() {
        out.push_str("\n### Specific Fixes Required\n");
        for ann in &fixes {
            out.push_str(&format!("- `{}`: {}\n", ann.line_ref, ann.comment));
        }
    }

    out
}

/// Generate Git Trailer format.
pub fn generate_git_trailers(state: &AppState, target_label: &str) -> String {
    let annotations = collect_guidance_annotations(state);
    if annotations.is_empty() {
        return "No annotations to export.".to_string();
    }

    let scores = collect_file_scores(state);

    let mut out = String::new();
    out.push_str(&format!(
        "Reviewed-by: reviewer via mdiff ({})\n",
        target_label
    ));

    let total_scores: Vec<u8> = scores.iter().map(|(_, s)| *s).collect();
    if !total_scores.is_empty() {
        let avg = total_scores.iter().map(|s| *s as f64).sum::<f64>() / total_scores.len() as f64;
        out.push_str(&format!("Review-score: {:.0}/5\n", avg));
    }

    for ann in &annotations {
        match classify_annotation(ann, &scores) {
            GuidanceSection::Critical => {
                let short_comment = truncate_for_trailer(&ann.comment);
                out.push_str(&format!(
                    "Review-critical: {} {}\n",
                    ann.line_ref, short_comment
                ));
            }
            GuidanceSection::Suggestion | GuidanceSection::StyleNote => {
                let short_comment = truncate_for_trailer(&ann.comment);
                out.push_str(&format!(
                    "Review-suggestion: {} {}\n",
                    ann.line_ref, short_comment
                ));
            }
            GuidanceSection::Positive => {
                let short_comment = truncate_for_trailer(&ann.comment);
                out.push_str(&format!(
                    "Review-approved: {} {}\n",
                    ann.line_ref, short_comment
                ));
            }
            GuidanceSection::Question => {
                let short_comment = truncate_for_trailer(&ann.comment);
                out.push_str(&format!(
                    "Review-question: {} {}\n",
                    ann.line_ref, short_comment
                ));
            }
        }
    }

    out
}

fn truncate_for_trailer(s: &str) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() > 72 {
        format!("{}...", &first_line[..69])
    } else {
        first_line.to_string()
    }
}

/// Generate a basic Markdown export of annotations.
fn generate_markdown(state: &AppState, target_label: &str) -> String {
    let export = build_export(state, target_label);
    let mut out = String::new();
    out.push_str(&format!("# Feedback for {}\n\n", target_label));
    out.push_str(&format!("*Exported at {}*\n\n", export.exported_at));
    out.push_str(&format!(
        "**Summary**: {} files, {} reviewed, {} annotations\n\n",
        export.summary.total_files, export.summary.files_reviewed, export.summary.total_annotations,
    ));

    for file in &export.files {
        if file.annotations.is_empty() {
            continue;
        }
        out.push_str(&format!("## `{}`\n\n", file.path));
        out.push_str(&format!(
            "+{} / -{} | Status: {}\n\n",
            file.additions, file.deletions, file.review_status
        ));
        for ann in &file.annotations {
            let range = match (ann.old_range, ann.new_range) {
                (_, Some((s, e))) if s == e => format!("Line {}", s),
                (_, Some((s, e))) => format!("Lines {}-{}", s, e),
                (Some((s, e)), None) if s == e => format!("Removed line {}", s),
                (Some((s, e)), None) => format!("Removed lines {}-{}", s, e),
                (None, None) => "Unknown range".to_string(),
            };
            out.push_str(&format!("- **{}**: {}\n", range, ann.comment));
        }
        out.push('\n');
    }

    out
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
        let result = export_feedback(&state, repo_path, &state.target_label);
        assert!(result.is_ok(), "Export should succeed");

        let path = result.unwrap();

        // Verify the file exists and contains valid JSON
        let content = std::fs::read_to_string(&path).expect("Should be able to read export file");
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("Should be valid JSON");

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
        assert!(!export.exported_at.is_empty());
        assert_eq!(export.summary.total_files, 0); // No deltas in empty state
        assert!(export.files.is_empty());
    }

    fn state_with_annotations() -> AppState {
        use crate::state::annotation_state::{
            Annotation, AnnotationCategory, AnnotationSeverity, LineAnchor,
        };
        let diff_options = DiffOptions::new(false, false);
        let theme = Theme::from_name("one-dark");
        let mut state = AppState::new(diff_options, theme);
        state.target_label = "main".to_string();

        state.annotations.add(Annotation {
            anchor: LineAnchor {
                file_path: "src/handler.rs".to_string(),
                old_range: None,
                new_range: Some((45, 52)),
            },
            comment: "unwrap() on user input will panic".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            category: AnnotationCategory::Bug,
            severity: AnnotationSeverity::Critical,
        });

        state.annotations.add(Annotation {
            anchor: LineAnchor {
                file_path: "src/db.rs".to_string(),
                old_range: None,
                new_range: Some((30, 45)),
            },
            comment: "N+1 query in user loading loop".to_string(),
            created_at: "2025-01-01T00:00:01Z".to_string(),
            category: AnnotationCategory::Performance,
            severity: AnnotationSeverity::Major,
        });

        state.annotations.add(Annotation {
            anchor: LineAnchor {
                file_path: "src/handler.rs".to_string(),
                old_range: None,
                new_range: Some((80, 80)),
            },
            comment: "Function exceeds 50 lines".to_string(),
            created_at: "2025-01-01T00:00:02Z".to_string(),
            category: AnnotationCategory::Style,
            severity: AnnotationSeverity::Minor,
        });

        state.annotations.add(Annotation {
            anchor: LineAnchor {
                file_path: "src/auth.rs".to_string(),
                old_range: None,
                new_range: Some((10, 10)),
            },
            comment: "Why is this endpoint public?".to_string(),
            created_at: "2025-01-01T00:00:03Z".to_string(),
            category: AnnotationCategory::Question,
            severity: AnnotationSeverity::Info,
        });

        state
    }

    #[test]
    fn test_agent_guidance_no_annotations() {
        let diff_options = DiffOptions::new(false, false);
        let theme = Theme::from_name("one-dark");
        let state = AppState::new(diff_options, theme);
        let result = generate_agent_guidance(&state, "HEAD");
        assert_eq!(result, "No annotations to export.");
    }

    #[test]
    fn test_agent_guidance_with_annotations() {
        let state = state_with_annotations();
        let result = generate_agent_guidance(&state, "main");

        assert!(result.contains("I reviewed your changes in main"));
        assert!(result.contains("## Critical Issues (must fix)"));
        assert!(result.contains("unwrap()"));
        assert!(result.contains("N+1 query"));
        assert!(result.contains("## Style Notes (low priority)"));
        assert!(result.contains("Function exceeds 50 lines"));
        assert!(result.contains("## Questions for Clarification"));
        assert!(result.contains("Why is this endpoint public?"));
    }

    #[test]
    fn test_claude_md_patch_with_annotations() {
        let state = state_with_annotations();
        let result = generate_claude_md_patch(&state, "main");

        assert!(result.contains("## Review Feedback (from mdiff review of main)"));
        assert!(result.contains("### Patterns to Avoid"));
        assert!(result.contains("unwrap()"));
        assert!(result.contains("### Specific Fixes Required"));
    }

    #[test]
    fn test_git_trailers_with_annotations() {
        let state = state_with_annotations();
        let result = generate_git_trailers(&state, "main");

        assert!(result.contains("Reviewed-by: reviewer via mdiff (main)"));
        assert!(result.contains("Review-critical:"));
        assert!(result.contains("Review-suggestion:"));
        assert!(result.contains("Review-question:"));
    }

    #[test]
    fn test_claude_md_patch_no_annotations() {
        let diff_options = DiffOptions::new(false, false);
        let theme = Theme::from_name("one-dark");
        let state = AppState::new(diff_options, theme);
        let result = generate_claude_md_patch(&state, "HEAD");
        assert_eq!(result, "No annotations to export.");
    }

    #[test]
    fn test_git_trailers_no_annotations() {
        let diff_options = DiffOptions::new(false, false);
        let theme = Theme::from_name("one-dark");
        let state = AppState::new(diff_options, theme);
        let result = generate_git_trailers(&state, "HEAD");
        assert_eq!(result, "No annotations to export.");
    }

    #[test]
    fn test_truncate_for_trailer_short() {
        assert_eq!(truncate_for_trailer("short text"), "short text");
    }

    #[test]
    fn test_truncate_for_trailer_long() {
        let long = "a".repeat(100);
        let result = truncate_for_trailer(&long);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 72 + 3);
    }

    #[test]
    fn test_export_format_enum() {
        assert_eq!(ExportFormat::Json.label(), "JSON");
        assert_eq!(ExportFormat::AgentGuidance.shortcut(), 'A');
        assert_eq!(ExportFormat::all().len(), 5);
    }
}
