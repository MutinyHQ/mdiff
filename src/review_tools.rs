//! Tools available to agentic review agents.
//!
//! Shared tools (parent + child): list_diff_files, read_diff, list_files,
//! read_file, search
//! Parent-only: annotate_file (spawns a child sub-agent)
//! Child-only: create_annotation (produces line-level annotation)

use crate::git::types::{DiffLineOrigin, FileDelta, FileStatus};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ================================================================
// Shared diff helpers
// ================================================================

pub const CHILD_INLINE_DIFF_MAX_BYTES: usize = 8 * 1024;

fn status_label(status: &FileStatus) -> &'static str {
    match status {
        FileStatus::Added => "A",
        FileStatus::Deleted => "D",
        FileStatus::Modified => "M",
        FileStatus::Renamed => "R",
        FileStatus::Untracked => "?",
    }
}

pub fn render_diff_text(deltas: &[FileDelta]) -> String {
    let mut out = String::new();
    for delta in deltas {
        if delta.binary {
            continue;
        }

        let old_path = delta
            .old_path
            .as_ref()
            .unwrap_or(&delta.path)
            .to_string_lossy();
        let new_path = delta.path.to_string_lossy();
        out.push_str(&format!("--- a/{old_path}\n+++ b/{new_path}\n"));
        for hunk in &delta.hunks {
            out.push_str(&hunk.header);
            if !hunk.header.ends_with('\n') {
                out.push('\n');
            }
            for line in &hunk.lines {
                let prefix = match line.origin {
                    DiffLineOrigin::Addition => "+",
                    DiffLineOrigin::Deletion => "-",
                    DiffLineOrigin::Context => " ",
                };
                out.push_str(&format!("{prefix}{}\n", line.content.trim_end()));
            }
        }
        out.push('\n');
    }
    out
}

pub fn render_file_diff(delta: &FileDelta) -> String {
    render_diff_text(std::slice::from_ref(delta))
}

pub fn diff_summary_line(delta: &FileDelta) -> String {
    let path = delta.path.to_string_lossy();
    let stats = format!("+{} -{}", delta.additions, delta.deletions);
    let rename = delta
        .old_path
        .as_ref()
        .map(|old| format!(" (from {})", old.to_string_lossy()))
        .unwrap_or_default();
    let binary = if delta.binary { " [binary]" } else { "" };

    format!(
        "- [{}] {}{} ({stats}){binary}",
        status_label(&delta.status),
        path,
        rename
    )
}

pub fn render_diff_summary(deltas: &[FileDelta]) -> String {
    deltas
        .iter()
        .map(diff_summary_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn binary_diff_message(delta: &FileDelta) -> String {
    let path = delta.path.to_string_lossy();
    format!(
        "Binary diff for {} [{}] (+{} -{}). Patch text is unavailable.",
        path,
        status_label(&delta.status),
        delta.additions,
        delta.deletions
    )
}

fn find_delta<'a>(deltas: &'a [FileDelta], file_path: &str) -> Option<&'a FileDelta> {
    deltas
        .iter()
        .find(|delta| delta.path.to_string_lossy() == file_path)
}

// ================================================================
// Shared tools: list_diff_files, read_diff, list_files, read_file, search
// ================================================================

/// List files in the current diff with status and stats.
#[derive(Debug, Clone)]
pub struct ListDiffFilesTool {
    pub deltas: Arc<Vec<FileDelta>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListDiffFilesArgs {}

#[derive(Debug, thiserror::Error)]
#[error("ListDiffFiles error: {0}")]
pub struct ListDiffFilesError(String);

impl Tool for ListDiffFilesTool {
    const NAME: &'static str = "list_diff_files";
    type Error = ListDiffFilesError;
    type Args = ListDiffFilesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_diff_files".to_string(),
            description: "List files in the current diff with status, rename info, additions, deletions, and binary flag.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let files: Vec<serde_json::Value> = self
            .deltas
            .iter()
            .map(|delta| {
                json!({
                    "path": delta.path.to_string_lossy().to_string(),
                    "old_path": delta.old_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    "status": status_label(&delta.status),
                    "additions": delta.additions,
                    "deletions": delta.deletions,
                    "binary": delta.binary,
                })
            })
            .collect();

        serde_json::to_string_pretty(&files)
            .map_err(|e| ListDiffFilesError(format!("Failed to serialize diff files: {e}")))
    }
}

/// Read a single file diff from the current diff.
#[derive(Debug, Clone)]
pub struct ReadDiffTool {
    pub deltas: Arc<Vec<FileDelta>>,
}

#[derive(Debug, Deserialize)]
pub struct ReadDiffArgs {
    /// File path relative to repo root from the current diff.
    pub file_path: String,
}

#[derive(Debug, thiserror::Error)]
#[error("ReadDiff error: {0}")]
pub struct ReadDiffError(String);

impl Tool for ReadDiffTool {
    const NAME: &'static str = "read_diff";
    type Error = ReadDiffError;
    type Args = ReadDiffArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_diff".to_string(),
            description: "Read the full unified diff for one changed file from the current diff."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Exact file path from list_diff_files"
                    }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let delta = find_delta(self.deltas.as_slice(), &args.file_path).ok_or_else(|| {
            ReadDiffError(format!(
                "File not found in current diff: {}",
                args.file_path
            ))
        })?;

        if delta.binary {
            return Ok(binary_diff_message(delta));
        }

        Ok(render_file_diff(delta))
    }
}

/// List files in the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesTool {
    pub repo_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesArgs {
    /// Optional subdirectory to list (relative to repo root). If omitted, lists repo root.
    pub path: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("ListFiles error: {0}")]
pub struct ListFilesError(String);

impl Tool for ListFilesTool {
    const NAME: &'static str = "list_files";
    type Error = ListFilesError;
    type Args = ListFilesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List files and directories at a given path in the repository. Returns one entry per line.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Subdirectory to list, relative to repo root. Omit for repo root."
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let target = match &args.path {
            Some(p) => self.repo_path.join(p),
            None => self.repo_path.clone(),
        };

        let entries = std::fs::read_dir(&target)
            .map_err(|e| ListFilesError(format!("Cannot read {}: {}", target.display(), e)))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ListFilesError(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry
                .file_type()
                .map_err(|e| ListFilesError(e.to_string()))?;
            if file_type.is_dir() {
                result.push(format!("{}/", name));
            } else {
                result.push(name);
            }
        }
        result.sort();
        Ok(result.join("\n"))
    }
}

/// Read file contents from the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileTool {
    pub repo_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ReadFileArgs {
    /// File path relative to repo root.
    pub path: String,
    /// Optional start line (1-indexed). If omitted, reads from beginning.
    pub start_line: Option<usize>,
    /// Optional end line (1-indexed, inclusive). If omitted, reads to end.
    pub end_line: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
#[error("ReadFile error: {0}")]
pub struct ReadFileError(String);

impl Tool for ReadFileTool {
    const NAME: &'static str = "read_file";
    type Error = ReadFileError;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description:
                "Read the contents of a file in the repository. Can read a specific line range."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to repo root"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Start line number (1-indexed). Omit to read from beginning."
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "End line number (1-indexed, inclusive). Omit to read to end."
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = self.repo_path.join(&args.path);
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| ReadFileError(format!("Cannot read {}: {}", args.path, e)))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = args.start_line.unwrap_or(1).saturating_sub(1);
        let end = args.end_line.unwrap_or(lines.len()).min(lines.len());

        if start >= lines.len() {
            return Ok(String::new());
        }

        let selected: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>4}\t{}", start + i + 1, line))
            .collect();

        Ok(selected.join("\n"))
    }
}

/// Search file contents using ripgrep (falls back to grep).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTool {
    pub repo_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct SearchArgs {
    /// Search pattern (regex supported).
    pub pattern: String,
    /// Optional file glob filter (e.g. "*.rs", "src/**/*.ts").
    pub glob: Option<String>,
    /// Maximum number of results to return. Defaults to 20.
    pub max_results: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
#[error("Search error: {0}")]
pub struct SearchError(String);

impl Tool for SearchTool {
    const NAME: &'static str = "search";
    type Error = SearchError;
    type Args = SearchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search".to_string(),
            description: "Search file contents in the repository using regex. Returns matching lines with file paths and line numbers.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex supported)"
                    },
                    "glob": {
                        "type": "string",
                        "description": "File glob filter (e.g. '*.rs', 'src/**/*.ts')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results. Defaults to 20."
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let max = args.max_results.unwrap_or(20);

        // Try ripgrep first, fall back to grep
        let output = try_ripgrep(&self.repo_path, &args.pattern, args.glob.as_deref(), max)
            .or_else(|_| try_grep(&self.repo_path, &args.pattern, max))
            .map_err(|e| SearchError(e.to_string()))?;

        Ok(output)
    }
}

fn try_ripgrep(
    repo_path: &PathBuf,
    pattern: &str,
    glob: Option<&str>,
    max: usize,
) -> Result<String, String> {
    let mut cmd = std::process::Command::new("rg");
    cmd.arg("--no-heading")
        .arg("--line-number")
        .arg("--color=never")
        .arg("--max-count")
        .arg(max.to_string())
        .arg(pattern)
        .arg(".")
        .current_dir(repo_path);

    if let Some(g) = glob {
        cmd.arg("--glob").arg(g);
    }

    let output = cmd.output().map_err(|e| format!("rg not found: {}", e))?;

    if output.status.success() || output.status.code() == Some(1) {
        // code 1 = no matches, which is fine
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn try_grep(repo_path: &PathBuf, pattern: &str, max: usize) -> Result<String, String> {
    let output = std::process::Command::new("grep")
        .arg("-rn")
        .arg("--color=never")
        .arg("-m")
        .arg(max.to_string())
        .arg(pattern)
        .arg(".")
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("grep failed: {}", e))?;

    if output.status.success() || output.status.code() == Some(1) {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ================================================================
// Parent-only tool: annotate_file
// ================================================================

/// Tool call result from annotate_file — captured by the pipeline to spawn child agents.
/// The tool itself doesn't execute anything; the pipeline intercepts the call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotateFileArgs {
    /// The file path this concern relates to (from the diff).
    pub file_path: String,
    /// Description of the concern.
    pub concern: String,
    /// Category: Bug, Style, Performance, Security, Suggestion, Question, or Nitpick.
    pub category: String,
    /// Severity: Critical, Major, Minor, or Info.
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotateFileTool;

#[derive(Debug, thiserror::Error)]
#[error("AnnotateFile error: {0}")]
pub struct AnnotateFileError(String);

impl Tool for AnnotateFileTool {
    const NAME: &'static str = "annotate_file";
    type Error = AnnotateFileError;
    type Args = AnnotateFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "annotate_file".to_string(),
            description: "Flag a concern on a specific file to spawn a sub-agent that will create a precise line-level annotation. Call this for each concern you identify during your review.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The exact file path from the diff"
                    },
                    "concern": {
                        "type": "string",
                        "description": "Clear description of the concern"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["Bug", "Style", "Performance", "Security", "Suggestion", "Question", "Nitpick"],
                        "description": "Category of the concern"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["Critical", "Major", "Minor", "Info"],
                        "description": "Severity of the concern"
                    }
                },
                "required": ["file_path", "concern", "category", "severity"]
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        // This tool is intercepted by the pipeline — it never actually executes.
        // The pipeline captures the args and spawns a child agent.
        Ok("Sub-agent dispatched for this concern.".to_string())
    }
}

// ================================================================
// Child-only tool: create_annotation
// ================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnotationArgs {
    /// The file path for this annotation.
    pub file_path: String,
    /// Start line in the old file (for deletions). Null if not applicable.
    pub old_line_start: Option<u32>,
    /// End line in the old file (for deletions). Null if not applicable.
    pub old_line_end: Option<u32>,
    /// Start line in the new file (for additions/modifications). Null if not applicable.
    pub new_line_start: Option<u32>,
    /// End line in the new file (for additions/modifications). Null if not applicable.
    pub new_line_end: Option<u32>,
    /// Category: Bug, Style, Performance, Security, Suggestion, Question, or Nitpick.
    pub category: String,
    /// Severity: Critical, Major, Minor, or Info.
    pub severity: String,
    /// The annotation comment — actionable and specific.
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct CreateAnnotationTool {
    pub sink: Arc<Mutex<Vec<CreateAnnotationArgs>>>,
}

#[derive(Debug, thiserror::Error)]
#[error("CreateAnnotation error: {0}")]
pub struct CreateAnnotationError(String);

impl Tool for CreateAnnotationTool {
    const NAME: &'static str = "create_annotation";
    type Error = CreateAnnotationError;
    type Args = CreateAnnotationArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "create_annotation".to_string(),
            description: "Create a precise line-level annotation on the diff. Use new_line_start/end for additions or modified lines (+ prefix in diff). Use old_line_start/end for deletions (- prefix in diff). Set the unused range to null.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The exact file path"
                    },
                    "old_line_start": {
                        "type": "integer",
                        "description": "Start line in old file (for deletions). Null if not applicable."
                    },
                    "old_line_end": {
                        "type": "integer",
                        "description": "End line in old file (for deletions). Null if not applicable."
                    },
                    "new_line_start": {
                        "type": "integer",
                        "description": "Start line in new file (for additions). Null if not applicable."
                    },
                    "new_line_end": {
                        "type": "integer",
                        "description": "End line in new file (for additions). Null if not applicable."
                    },
                    "category": {
                        "type": "string",
                        "enum": ["Bug", "Style", "Performance", "Security", "Suggestion", "Question", "Nitpick"]
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["Critical", "Major", "Minor", "Info"]
                    },
                    "comment": {
                        "type": "string",
                        "description": "Actionable annotation comment"
                    }
                },
                "required": ["file_path", "category", "severity", "comment"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.sink
            .lock()
            .map_err(|e| CreateAnnotationError(format!("Lock poisoned: {}", e)))?
            .push(args);
        Ok("Annotation created.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::{DiffLine, Hunk};
    use std::path::PathBuf;

    fn sample_delta() -> FileDelta {
        FileDelta {
            path: PathBuf::from("src/lib.rs"),
            old_path: None,
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                header: "@@ -1,2 +1,2 @@".to_string(),
                lines: vec![
                    DiffLine {
                        origin: DiffLineOrigin::Context,
                        old_lineno: Some(1),
                        new_lineno: Some(1),
                        content: "fn old() {".to_string(),
                    },
                    DiffLine {
                        origin: DiffLineOrigin::Deletion,
                        old_lineno: Some(2),
                        new_lineno: None,
                        content: "    old_call();".to_string(),
                    },
                    DiffLine {
                        origin: DiffLineOrigin::Addition,
                        old_lineno: None,
                        new_lineno: Some(2),
                        content: "    new_call();".to_string(),
                    },
                ],
            }],
            additions: 1,
            deletions: 1,
            binary: false,
        }
    }

    #[test]
    fn render_diff_summary_includes_rename_and_binary() {
        let mut renamed = sample_delta();
        renamed.status = FileStatus::Renamed;
        renamed.old_path = Some(PathBuf::from("src/old.rs"));
        renamed.path = PathBuf::from("src/new.rs");

        let mut binary = sample_delta();
        binary.path = PathBuf::from("assets/logo.png");
        binary.status = FileStatus::Added;
        binary.binary = true;
        binary.hunks.clear();
        binary.additions = 0;
        binary.deletions = 0;

        let summary = render_diff_summary(&[renamed, binary]);
        assert!(summary.contains("[R] src/new.rs (from src/old.rs) (+1 -1)"));
        assert!(summary.contains("[A] assets/logo.png (+0 -0) [binary]"));
    }

    #[tokio::test]
    async fn read_diff_tool_returns_patch_for_changed_file() {
        let tool = ReadDiffTool {
            deltas: Arc::new(vec![sample_delta()]),
        };

        let output = tool
            .call(ReadDiffArgs {
                file_path: "src/lib.rs".to_string(),
            })
            .await
            .expect("read diff should succeed");

        assert!(output.contains("--- a/src/lib.rs"));
        assert!(output.contains("+++ b/src/lib.rs"));
        assert!(output.contains("+    new_call();"));
    }

    #[tokio::test]
    async fn read_diff_tool_handles_binary_and_missing_files() {
        let mut binary = sample_delta();
        binary.path = PathBuf::from("assets/logo.png");
        binary.binary = true;
        binary.hunks.clear();

        let tool = ReadDiffTool {
            deltas: Arc::new(vec![binary]),
        };

        let output = tool
            .call(ReadDiffArgs {
                file_path: "assets/logo.png".to_string(),
            })
            .await
            .expect("binary diff should return a message");
        assert!(output.contains("Binary diff for assets/logo.png"));

        let err = tool
            .call(ReadDiffArgs {
                file_path: "missing.rs".to_string(),
            })
            .await
            .expect_err("missing file should error");
        assert!(err.to_string().contains("File not found in current diff"));
    }
}
