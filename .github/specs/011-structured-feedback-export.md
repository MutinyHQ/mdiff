# Spec: Structured Feedback Export (`Ctrl+E`)

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Medium (4-6 files changed)

## Problem

mdiff's core value proposition is providing structured human feedback on coding agent output. Currently, the only way to consume review feedback is through the prompt preview (`p` key) which generates a human-readable text block, or by copying the prompt to clipboard (`y` key). Neither format is machine-parseable.

When reviewers finish annotating an agent's changeset, there's no way to export the structured feedback (annotations, line scores, review status, checklist state) in a format that downstream tools — CI pipelines, agent training loops, RLHF data collection systems, or the coding agent itself — can programmatically consume.

This is a critical gap: the entire agentic AI ecosystem is moving toward structured decision trajectories for training data (not free-text labels), and mdiff is sitting on exactly that data but has no export path.

## Solution

### New Action: `ExportFeedback` triggered by `Ctrl+E`

Add a structured feedback export system that serializes the entire review session into JSON (primary) and optionally YAML formats, written to a `.mdiff-feedback/` directory in the repo root.

### Export Schema

```json
{
  "version": 1,
  "exported_at": "2026-03-06T15:30:00Z",
  "target": "HEAD",
  "summary": {
    "total_files": 15,
    "files_reviewed": 12,
    "files_with_annotations": 5,
    "total_annotations": 8,
    "total_line_scores": 23,
    "review_completeness": 0.8
  },
  "files": [
    {
      "path": "src/app.rs",
      "review_status": "reviewed",
      "additions": 45,
      "deletions": 12,
      "annotations": [
        {
          "type": "comment",
          "old_range": null,
          "new_range": [10, 15],
          "comment": "This error handling should use ? instead of unwrap",
          "category": "bug",
          "severity": "major"
        }
      ],
      "line_scores": [
        {
          "line": 42,
          "score": 2,
          "side": "new"
        }
      ]
    }
  ],
  "decision": null
}
```

### Implementation Details

#### File: `src/action.rs`

Add new action variant:

```rust
// Feedback export
ExportFeedback,
```

#### File: `src/event.rs`

Map `Ctrl+E` to the new action in the global bindings (Priority 6):

```rust
KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    return Some(Action::ExportFeedback)
}
```

#### File: `src/export.rs` (NEW)

New module containing the export logic:

```rust
use serde::Serialize;
use std::path::{Path, PathBuf};
use chrono::Utc;

use crate::state::AppState;
use crate::state::review_state::FileReviewStatus;

#[derive(Serialize)]
pub struct FeedbackExport {
    pub version: u32,
    pub exported_at: String,
    pub target: String,
    pub summary: FeedbackSummary,
    pub files: Vec<FileFeedback>,
}

#[derive(Serialize)]
pub struct FeedbackSummary {
    pub total_files: usize,
    pub files_reviewed: usize,
    pub files_with_annotations: usize,
    pub total_annotations: usize,
    pub review_completeness: f64,
}

#[derive(Serialize)]
pub struct FileFeedback {
    pub path: String,
    pub review_status: String,
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
}

#[derive(Serialize)]
pub struct LineScoreExport {
    pub line: u32,
    pub score: u8,
    pub side: String,
}

pub fn export_feedback(state: &AppState, repo_path: &Path, target_label: &str) -> Result<PathBuf, String> {
    let export = build_export(state, target_label);
    
    let dir = repo_path.join(".mdiff-feedback");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("feedback_{}_{}.json", target_label, timestamp);
    let path = dir.join(&filename);
    
    let json = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    
    Ok(path)
}

fn build_export(state: &AppState, target_label: &str) -> FeedbackExport {
    // Iterate over deltas, collect annotations per file, review status, line scores
    // ... implementation details
}
```

#### File: `src/app.rs`

Handle the new action in the action dispatch:

```rust
Action::ExportFeedback => {
    match export::export_feedback(&self.state, &self.repo_path, &self.state.target_label) {
        Ok(path) => {
            self.state.status_message = Some((
                format!("Feedback exported to {}", path.display()),
                false,
            ));
        }
        Err(e) => {
            self.state.status_message = Some((
                format!("Export failed: {}", e),
                true,
            ));
        }
    }
    self.hud_collapse_countdown = 100;
}
```

#### File: `src/main.rs`

Add `mod export;` declaration.

### Dependencies

- `serde` and `serde_json` are already in Cargo.toml
- `chrono` may need to be added for timestamp formatting (or use `std::time::SystemTime` to avoid new dependencies)

### Avoiding new dependencies

To avoid adding `chrono`, use:
```rust
use std::time::SystemTime;
let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

And format the exported_at field as a Unix timestamp or use a simple date formatter.

### Gitignore

Add `.mdiff-feedback/` to the `.gitignore` ensure-pattern logic in `src/session.rs` (similar to how `.mdiff-session/` is handled), so exported feedback files aren't accidentally committed.

## Files Changed

- `src/action.rs` — Add `ExportFeedback` variant
- `src/event.rs` — Map `Ctrl+E` to `ExportFeedback`
- `src/export.rs` — New module with serialization logic
- `src/app.rs` — Handle `ExportFeedback` action
- `src/main.rs` — Add `mod export`
- `src/components/which_key.rs` — Add `Ctrl+E` to help entries

## User Flow

1. User reviews agent changeset, leaving annotations and scores
2. User presses `Ctrl+E`
3. Status bar shows: "Feedback exported to .mdiff-feedback/feedback_HEAD_20260306_153000.json"
4. JSON file contains all structured feedback, ready for agent consumption
5. CI pipeline or agent wrapper can read the file and feed it back to the agent

## Testing

1. `cargo check` passes
2. Open mdiff on a repo, add some annotations
3. Press `Ctrl+E` — verify JSON file is created
4. Verify JSON is valid and contains all annotations, scores, and review status
5. Verify `.mdiff-feedback/` directory is gitignored
