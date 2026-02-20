use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::state::annotation_state::Annotation;
use crate::state::AnnotationState;

#[derive(Serialize, Deserialize)]
struct SessionFile {
    version: u32,
    target_label: String,
    annotations: Vec<AnnotationEntry>,
}

/// V2 annotation entry with separate old/new ranges.
#[derive(Serialize, Deserialize)]
struct AnnotationEntry {
    file_path: String,
    #[serde(default)]
    old_start: Option<u32>,
    #[serde(default)]
    old_end: Option<u32>,
    #[serde(default)]
    new_start: Option<u32>,
    #[serde(default)]
    new_end: Option<u32>,
    // V1 compat fields (only present in v1 files)
    #[serde(default)]
    line_start: Option<u32>,
    #[serde(default)]
    line_end: Option<u32>,
    comment: String,
    created_at: String,
}

fn session_dir(repo_path: &Path) -> PathBuf {
    repo_path.join(".mdiff")
}

fn session_file(repo_path: &Path, target_label: &str) -> PathBuf {
    let sanitized = target_label.replace(['/', '\\', ':', ' '], "_");
    session_dir(repo_path).join(format!("session_{sanitized}.json"))
}

/// Ensure `.mdiff/` is listed in `.gitignore`.
fn ensure_gitignore(repo_path: &Path) {
    let gitignore_path = repo_path.join(".gitignore");
    let entry = ".mdiff/";

    if let Ok(contents) = fs::read_to_string(&gitignore_path) {
        if contents.lines().any(|line| line.trim() == entry) {
            return;
        }
        // Append to existing .gitignore
        if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&gitignore_path) {
            // Add newline if file doesn't end with one
            if !contents.ends_with('\n') {
                let _ = writeln!(f);
            }
            let _ = writeln!(f, "{entry}");
        }
    } else {
        // No .gitignore yet — create one
        let _ = fs::write(&gitignore_path, format!("{entry}\n"));
    }
}

/// Load annotations from the session file, if it exists and matches the target.
/// Accepts both v1 (line_start/line_end) and v2 (old/new ranges) formats.
pub fn load_session(repo_path: &Path, target_label: &str) -> AnnotationState {
    let path = session_file(repo_path, target_label);
    let mut state = AnnotationState::default();

    let Ok(contents) = fs::read_to_string(&path) else {
        return state;
    };

    let Ok(session) = serde_json::from_str::<SessionFile>(&contents) else {
        return state;
    };

    if !(session.version == 1 || session.version == 2) || session.target_label != target_label {
        return state;
    }

    for entry in session.annotations {
        let (old_range, new_range) = if session.version == 1 {
            // Migrate v1: line_start/line_end → new_range (best guess)
            let ls = entry.line_start.unwrap_or(1);
            let le = entry.line_end.unwrap_or(ls);
            (None, Some((ls, le)))
        } else {
            // V2: use explicit old/new ranges
            let old_range = entry.old_start.zip(entry.old_end);
            let new_range = entry.new_start.zip(entry.new_end);
            (old_range, new_range)
        };

        state.add(Annotation {
            anchor: crate::state::annotation_state::LineAnchor {
                file_path: entry.file_path,
                old_range,
                new_range,
            },
            comment: entry.comment,
            created_at: entry.created_at,
        });
    }

    state
}

/// Save annotations to the session file (always v2 format).
pub fn save_session(repo_path: &Path, target_label: &str, annotations: &AnnotationState) {
    let dir = session_dir(repo_path);
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    ensure_gitignore(repo_path);

    let entries: Vec<AnnotationEntry> = annotations
        .all_sorted()
        .into_iter()
        .map(|a| AnnotationEntry {
            file_path: a.anchor.file_path.clone(),
            old_start: a.anchor.old_range.map(|(s, _)| s),
            old_end: a.anchor.old_range.map(|(_, e)| e),
            new_start: a.anchor.new_range.map(|(s, _)| s),
            new_end: a.anchor.new_range.map(|(_, e)| e),
            line_start: None,
            line_end: None,
            comment: a.comment.clone(),
            created_at: a.created_at.clone(),
        })
        .collect();

    let session = SessionFile {
        version: 2,
        target_label: target_label.to_string(),
        annotations: entries,
    };

    if let Ok(json) = serde_json::to_string_pretty(&session) {
        let _ = fs::write(session_file(repo_path, target_label), json);
    }
}
