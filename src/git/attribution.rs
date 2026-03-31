use anyhow::Result;
use git2::Repository;

use crate::git::types::ComparisonTarget;
use crate::state::timeline_state::TimelineCommit;

/// Collect commits in the diff range, returning them in chronological order
/// (oldest first). For Branch/Commit targets this walks from HEAD back to the
/// merge-base. For HeadVsWorkdir it returns the single HEAD commit (if any).
pub fn collect_timeline_commits(
    repo: &Repository,
    target: &ComparisonTarget,
) -> Result<Vec<TimelineCommit>> {
    let head_commit = match repo.head() {
        Ok(head) => head.peel_to_commit()?.id(),
        Err(_) => return Ok(Vec::new()),
    };

    let base_oid = match target {
        ComparisonTarget::HeadVsWorkdir => {
            return Ok(vec![commit_to_timeline(repo, head_commit)?]);
        }
        ComparisonTarget::Branch(name) => {
            let obj = repo.revparse_single(name)?;
            let target_commit = obj.peel_to_commit()?;
            match repo.merge_base(head_commit, target_commit.id()) {
                Ok(base) => base,
                Err(_) => target_commit.id(),
            }
        }
        ComparisonTarget::Commit(oid) => match repo.merge_base(head_commit, *oid) {
            Ok(base) => base,
            Err(_) => *oid,
        },
    };

    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_commit)?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;

    let mut commits = Vec::new();
    for oid_result in revwalk {
        let oid = oid_result?;
        if oid == base_oid {
            continue;
        }
        let commit = repo.find_commit(oid)?;
        if commit.parent_ids().any(|parent_id| parent_id == base_oid)
            || commits
                .iter()
                .any(|c: &TimelineCommit| commit.parent_ids().any(|pid| pid.to_string() == c.oid))
            || commits.is_empty()
        {
            commits.push(commit_to_timeline(repo, oid)?);
        } else {
            let is_descendant = commit.parent_ids().any(|pid| {
                commits
                    .iter()
                    .any(|c: &TimelineCommit| c.oid == pid.to_string())
            });
            if !is_descendant {
                // Commit is still after base but not directly chained; include it
                // if it's reachable from base (revwalk already ensures this).
                commits.push(commit_to_timeline(repo, oid)?);
            }
        }
    }

    Ok(commits)
}

fn commit_to_timeline(repo: &Repository, oid: git2::Oid) -> Result<TimelineCommit> {
    let commit = repo.find_commit(oid)?;
    let summary = commit.summary().unwrap_or("<no message>").to_string();
    let author = commit.author().name().unwrap_or("unknown").to_string();
    let time = commit.time();
    let timestamp = format_timestamp(time.seconds());

    let (files_changed, additions, deletions) = commit_stats(repo, &commit)?;

    Ok(TimelineCommit {
        oid: oid.to_string(),
        summary,
        author,
        timestamp,
        files_changed,
        additions,
        deletions,
    })
}

fn commit_stats(repo: &Repository, commit: &git2::Commit<'_>) -> Result<(usize, usize, usize)> {
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let stats = diff.stats()?;

    Ok((stats.files_changed(), stats.insertions(), stats.deletions()))
}

fn format_timestamp(seconds: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let delta = now - seconds;
    if delta < 60 {
        "just now".to_string()
    } else if delta < 3600 {
        format!("{}m ago", delta / 60)
    } else if delta < 86400 {
        format!("{}h ago", delta / 3600)
    } else {
        format!("{}d ago", delta / 86400)
    }
}
