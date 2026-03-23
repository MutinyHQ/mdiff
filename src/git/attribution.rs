use anyhow::Result;
use git2::Repository;
use std::collections::HashMap;

use crate::git::types::{ComparisonTarget, FileDelta};
use crate::state::attribution_state::{AgentSession, AttributionState};

/// Build attribution state by analyzing commits between the diff target and HEAD.
///
/// Each commit between the base and HEAD becomes an agent session. Hunks are
/// attributed to the commit that introduced them by comparing each commit's
/// diff against the final combined diff.
pub fn build_attribution(
    repo: &Repository,
    target: &ComparisonTarget,
    deltas: &[FileDelta],
) -> Result<AttributionState> {
    let mut state = AttributionState::default();

    let commits = collect_commits(repo, target)?;
    if commits.is_empty() {
        return Ok(state);
    }

    for (idx, (oid, summary)) in commits.iter().enumerate() {
        let label = if summary.is_empty() {
            format!("Session #{}", idx + 1)
        } else {
            let sha_prefix = format!("{:.7}", oid);
            let truncated: String = summary.chars().take(40).collect();
            format!("{} {}", sha_prefix, truncated)
        };
        state.sessions.push(AgentSession {
            label,
            id: format!("{}", oid),
            color_index: idx as u8,
        });
    }

    // Attribute each file's hunks to the commit that most likely introduced them.
    // Strategy: walk commits oldest→newest, compute per-commit changed files/lines,
    // then assign hunks to the latest commit that touched them.
    let commit_file_sets = build_commit_file_sets(repo, &commits)?;

    for delta in deltas {
        let file_path = delta.path.to_string_lossy().to_string();
        for (hunk_idx, _hunk) in delta.hunks.iter().enumerate() {
            if let Some(session) =
                find_attributing_session(&file_path, &commits, &commit_file_sets, &state.sessions)
            {
                state
                    .hunk_attributions
                    .insert((file_path.clone(), hunk_idx), session);
            }
        }
    }

    // Add "Working Directory" session for uncommitted changes
    if matches!(target, ComparisonTarget::HeadVsWorkdir) {
        let wd_session = AgentSession {
            label: "Working Directory".to_string(),
            id: "workdir".to_string(),
            color_index: commits.len() as u8,
        };
        state.sessions.push(wd_session);
    }

    state.active = !state.sessions.is_empty();
    Ok(state)
}

/// Collect commits between the diff base and HEAD, ordered oldest to newest.
fn collect_commits(
    repo: &Repository,
    target: &ComparisonTarget,
) -> Result<Vec<(git2::Oid, String)>> {
    let head_oid = match repo.head() {
        Ok(head) => head.peel_to_commit()?.id(),
        Err(_) => return Ok(Vec::new()),
    };

    let base_oid = match target {
        ComparisonTarget::HeadVsWorkdir => return Ok(Vec::new()),
        ComparisonTarget::Branch(name) => {
            let obj = repo.revparse_single(name)?;
            let target_commit = obj.peel_to_commit()?;
            match repo.merge_base(head_oid, target_commit.id()) {
                Ok(base) => base,
                Err(_) => target_commit.id(),
            }
        }
        ComparisonTarget::Commit(oid) => match repo.merge_base(head_oid, *oid) {
            Ok(base) => base,
            Err(_) => *oid,
        },
    };

    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_oid)?;
    revwalk.hide(base_oid)?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;

    let mut commits = Vec::new();
    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        let summary = commit.summary().unwrap_or("").to_string();
        commits.push((oid, summary));
    }

    Ok(commits)
}

/// For each commit, collect the set of files it modified.
fn build_commit_file_sets(
    repo: &Repository,
    commits: &[(git2::Oid, String)],
) -> Result<Vec<Vec<String>>> {
    let mut result = Vec::new();

    for (oid, _) in commits {
        let commit = repo.find_commit(*oid)?;
        let tree = commit.tree()?;

        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut files = Vec::new();
        for delta_idx in 0..diff.deltas().len() {
            if let Some(delta) = diff.get_delta(delta_idx) {
                if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
        result.push(files);
    }

    Ok(result)
}

/// Find the session that should be attributed for a given file path.
/// Returns the latest commit session that touched the file.
fn find_attributing_session(
    file_path: &str,
    _commits: &[(git2::Oid, String)],
    commit_file_sets: &[Vec<String>],
    sessions: &[AgentSession],
) -> Option<AgentSession> {
    // Walk commits newest→oldest and return the first (latest) match
    for (idx, files) in commit_file_sets.iter().enumerate().rev() {
        if files.iter().any(|f| f == file_path) {
            return sessions.get(idx).cloned();
        }
    }
    None
}

/// Convenience function to compute per-file session breakdown for the navigator.
#[allow(dead_code)]
pub fn file_session_summary(
    state: &AttributionState,
    deltas: &[FileDelta],
) -> HashMap<String, Vec<AgentSession>> {
    let mut result: HashMap<String, Vec<AgentSession>> = HashMap::new();

    for delta in deltas {
        let file_path = delta.path.to_string_lossy().to_string();
        let mut sessions_for_file = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for (hunk_idx, _) in delta.hunks.iter().enumerate() {
            if let Some(session) = state.session_for_hunk(&file_path, hunk_idx) {
                if seen_ids.insert(session.id.clone()) {
                    sessions_for_file.push(session.clone());
                }
            }
        }

        if !sessions_for_file.is_empty() {
            result.insert(file_path, sessions_for_file);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_attributing_session_last_touch() {
        let commits = vec![
            (git2::Oid::zero(), "first".to_string()),
            (git2::Oid::zero(), "second".to_string()),
        ];
        let file_sets = vec![
            vec!["a.rs".to_string(), "b.rs".to_string()],
            vec!["b.rs".to_string(), "c.rs".to_string()],
        ];
        let sessions = vec![
            AgentSession {
                label: "S1".to_string(),
                id: "s1".to_string(),
                color_index: 0,
            },
            AgentSession {
                label: "S2".to_string(),
                id: "s2".to_string(),
                color_index: 1,
            },
        ];

        // "a.rs" only in commit 0 => S1
        let result = find_attributing_session("a.rs", &commits, &file_sets, &sessions);
        assert_eq!(result.unwrap().id, "s1");

        // "b.rs" in both commits => latest wins => S2
        let result = find_attributing_session("b.rs", &commits, &file_sets, &sessions);
        assert_eq!(result.unwrap().id, "s2");

        // "c.rs" only in commit 1 => S2
        let result = find_attributing_session("c.rs", &commits, &file_sets, &sessions);
        assert_eq!(result.unwrap().id, "s2");

        // "d.rs" not found
        let result = find_attributing_session("d.rs", &commits, &file_sets, &sessions);
        assert!(result.is_none());
    }
}
