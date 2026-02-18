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

#[derive(Serialize, Deserialize)]
struct AnnotationEntry {
    file_path: String,
    line_start: u32,
    line_end: u32,
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
pub fn load_session(repo_path: &Path, target_label: &str) -> AnnotationState {
    let path = session_file(repo_path, target_label);
    let mut state = AnnotationState::default();

    let Ok(contents) = fs::read_to_string(&path) else {
        return state;
    };

    let Ok(session) = serde_json::from_str::<SessionFile>(&contents) else {
        return state;
    };

    if session.version != 1 || session.target_label != target_label {
        return state;
    }

    for entry in session.annotations {
        state.add(Annotation {
            anchor: crate::state::annotation_state::LineAnchor {
                file_path: entry.file_path,
                line_start: entry.line_start,
                line_end: entry.line_end,
            },
            comment: entry.comment,
            created_at: entry.created_at,
        });
    }

    state
}

/// Save annotations to the session file.
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
            line_start: a.anchor.line_start,
            line_end: a.anchor.line_end,
            comment: a.comment.clone(),
            created_at: a.created_at.clone(),
        })
        .collect();

    let session = SessionFile {
        version: 1,
        target_label: target_label.to_string(),
        annotations: entries,
    };

    if let Ok(json) = serde_json::to_string_pretty(&session) {
        let _ = fs::write(session_file(repo_path, target_label), json);
    }
}
