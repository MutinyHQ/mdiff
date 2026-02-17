use anyhow::{Context, Result};
use git2::Repository;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub head_ref: Option<String>,
    pub is_main: bool,
    pub is_dirty: bool,
    pub agent: Option<AgentInfo>,
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub agent_type: AgentType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentType {
    ClaudeCode,
    Cursor,
    Aider,
    Copilot,
    Other,
}

impl AgentType {
    pub fn label(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude",
            AgentType::Cursor => "Cursor",
            AgentType::Aider => "Aider",
            AgentType::Copilot => "Copilot",
            AgentType::Other => "AI",
        }
    }
}

/// List all worktrees for the repository at the given path.
pub fn list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeInfo>> {
    let repo = Repository::discover(repo_path).context("Not a git repository")?;

    let mut worktrees = Vec::new();

    // Add the main worktree
    if let Some(workdir) = repo.workdir() {
        let head_ref = repo.head().ok().and_then(|h| {
            if h.is_branch() {
                h.shorthand().map(|s| s.to_string())
            } else {
                h.target().map(|oid| format!("{:.7}", oid))
            }
        });

        let is_dirty = repo_is_dirty(&repo);

        let mut info = WorktreeInfo {
            name: "main".to_string(),
            path: workdir.to_path_buf(),
            head_ref,
            is_main: true,
            is_dirty,
            agent: None,
        };
        info.agent = detect_agent(&info.path);
        worktrees.push(info);
    }

    // List linked worktrees
    let wt_names = repo.worktrees()?;
    for name in wt_names.iter() {
        let Some(name) = name else { continue };
        let Ok(wt) = repo.find_worktree(name) else {
            continue;
        };

        let wt_path = wt.path().to_path_buf();

        // Open the worktree's repo to get head info
        let (head_ref, is_dirty) = match Repository::open(&wt_path) {
            Ok(wt_repo) => {
                let head = wt_repo.head().ok().and_then(|h| {
                    if h.is_branch() {
                        h.shorthand().map(|s| s.to_string())
                    } else {
                        h.target().map(|oid| format!("{:.7}", oid))
                    }
                });
                let dirty = repo_is_dirty(&wt_repo);
                (head, dirty)
            }
            Err(_) => (None, false),
        };

        let mut info = WorktreeInfo {
            name: name.to_string(),
            path: wt_path,
            head_ref,
            is_main: false,
            is_dirty,
            agent: None,
        };
        info.agent = detect_agent(&info.path);
        worktrees.push(info);
    }

    Ok(worktrees)
}

fn repo_is_dirty(repo: &Repository) -> bool {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(false);
    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => !statuses.is_empty(),
        Err(_) => false,
    }
}

/// Detect if an AI agent is operating in this worktree directory.
fn detect_agent(path: &Path) -> Option<AgentInfo> {
    // Check for Claude Code markers
    if path.join(".claude").is_dir() {
        return Some(AgentInfo {
            agent_type: AgentType::ClaudeCode,
        });
    }

    // Check for Cursor markers
    if path.join(".cursorrules").is_file() || path.join(".cursor").is_dir() {
        return Some(AgentInfo {
            agent_type: AgentType::Cursor,
        });
    }

    // Check for Aider markers
    if path.join(".aider.conf.yml").is_file() || path.join(".aider").is_dir() {
        return Some(AgentInfo {
            agent_type: AgentType::Aider,
        });
    }

    // Check for Copilot markers
    if path.join(".github/copilot").is_dir() {
        return Some(AgentInfo {
            agent_type: AgentType::Copilot,
        });
    }

    // Fallback: check directory/branch naming patterns
    let dir_name = path.file_name()?.to_string_lossy().to_lowercase();
    for keyword in &["claude", "cursor", "aider", "copilot", "agent"] {
        if dir_name.contains(keyword) {
            let agent_type = match *keyword {
                "claude" => AgentType::ClaudeCode,
                "cursor" => AgentType::Cursor,
                "aider" => AgentType::Aider,
                "copilot" => AgentType::Copilot,
                _ => AgentType::Other,
            };
            return Some(AgentInfo {
                agent_type,
            });
        }
    }

    None
}
