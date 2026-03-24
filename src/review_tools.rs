//! Tools available to agentic review agents.
//!
//! Shared tools (parent + child): list_files, read_file, search
//! Parent-only: annotate_file (spawns a child sub-agent)
//! Child-only: create_annotation (produces line-level annotation)

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ================================================================
// Shared tools: list_files, read_file, search
// ================================================================

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
            parameters: serde_json::json!({
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
            parameters: serde_json::json!({
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
            parameters: serde_json::json!({
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
            parameters: serde_json::json!({
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
            parameters: serde_json::json!({
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
