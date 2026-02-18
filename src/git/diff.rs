use anyhow::{Context, Result};
use git2::{Delta, Diff, DiffFormat, DiffOptions, Repository};

use super::types::*;
use crate::state::diff_state::DiffOptions as AppDiffOptions;

pub struct DiffEngine;

impl DiffEngine {
    pub fn compute_diff(
        repo: &Repository,
        target: &ComparisonTarget,
        options: &AppDiffOptions,
    ) -> Result<Vec<FileDelta>> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.ignore_whitespace(options.ignore_whitespace);
        diff_opts.include_untracked(true);
        diff_opts.recurse_untracked_dirs(true);
        diff_opts.show_untracked_content(true);
        diff_opts.context_lines(999_999);

        let diff = match target {
            ComparisonTarget::HeadVsWorkdir => {
                // Get HEAD tree, if it exists (new repos may have no commits)
                let head_tree = match repo.head() {
                    Ok(head) => {
                        let commit = head.peel_to_commit()?;
                        Some(commit.tree()?)
                    }
                    Err(_) => None,
                };
                repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_opts))?
            }
            ComparisonTarget::Branch(name) => {
                let obj = repo
                    .revparse_single(name)
                    .with_context(|| format!("Could not resolve: {name}"))?;
                let target_commit = obj
                    .peel_to_commit()
                    .with_context(|| format!("{name} does not point to a commit"))?;
                let base_tree = Self::merge_base_tree(repo, target_commit.id())?;
                repo.diff_tree_to_workdir_with_index(Some(&base_tree), Some(&mut diff_opts))?
            }
            ComparisonTarget::Commit(oid) => {
                let base_tree = Self::merge_base_tree(repo, *oid)?;
                repo.diff_tree_to_workdir_with_index(Some(&base_tree), Some(&mut diff_opts))?
            }
        };

        Self::parse_diff(&diff)
    }

    /// Find the merge-base between HEAD and the given commit, returning the
    /// merge-base's tree. This implements 3-dot diff semantics: showing only
    /// the changes on the current branch since it diverged from the target.
    /// Falls back to the target commit's tree if HEAD doesn't exist or no
    /// merge-base is found.
    fn merge_base_tree(repo: &Repository, target_oid: git2::Oid) -> Result<git2::Tree<'_>> {
        let head_oid = match repo.head() {
            Ok(head) => head.peel_to_commit()?.id(),
            Err(_) => {
                // No HEAD (empty repo) — fall back to target tree directly
                return Ok(repo.find_commit(target_oid)?.tree()?);
            }
        };
        match repo.merge_base(head_oid, target_oid) {
            Ok(base_oid) => {
                let base_commit = repo.find_commit(base_oid)?;
                Ok(base_commit.tree()?)
            }
            Err(_) => {
                // No common ancestor — fall back to target tree
                Ok(repo.find_commit(target_oid)?.tree()?)
            }
        }
    }

    fn parse_diff(diff: &Diff<'_>) -> Result<Vec<FileDelta>> {
        let mut deltas: Vec<FileDelta> = Vec::new();

        let num_deltas = diff.deltas().len();
        for i in 0..num_deltas {
            let Some(delta) = diff.get_delta(i) else {
                continue;
            };
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .unwrap_or_else(|| std::path::Path::new("<unknown>"))
                .to_path_buf();

            let old_path = if delta.status() == Delta::Renamed {
                delta.old_file().path().map(|p| p.to_path_buf())
            } else {
                None
            };

            let status = match delta.status() {
                Delta::Added => FileStatus::Added,
                Delta::Deleted => FileStatus::Deleted,
                Delta::Modified => FileStatus::Modified,
                Delta::Renamed => FileStatus::Renamed,
                Delta::Untracked => FileStatus::Untracked,
                _ => FileStatus::Modified,
            };

            let binary = delta.flags().is_binary();

            deltas.push(FileDelta {
                path,
                old_path,
                status,
                hunks: Vec::new(),
                additions: 0,
                deletions: 0,
                binary,
            });
        }

        // Now parse the actual diff content using print
        let mut current_delta_idx: Option<usize> = None;
        let mut current_hunk: Option<Hunk> = None;

        diff.print(DiffFormat::Patch, |delta, hunk, line| {
            let delta_path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .unwrap_or_else(|| std::path::Path::new("<unknown>"));

            let idx = if let Some(i) = current_delta_idx {
                if deltas[i].path == delta_path {
                    i
                } else {
                    if let Some(h) = current_hunk.take() {
                        deltas[i].hunks.push(h);
                    }
                    deltas
                        .iter()
                        .position(|d| d.path == delta_path)
                        .unwrap_or(0)
                }
            } else {
                deltas
                    .iter()
                    .position(|d| d.path == delta_path)
                    .unwrap_or(0)
            };
            current_delta_idx = Some(idx);

            match line.origin() {
                'H' => {
                    if let Some(h) = current_hunk.take() {
                        deltas[idx].hunks.push(h);
                    }
                    let header = if let Some(ref h) = hunk {
                        format!(
                            "@@ -{},{} +{},{} @@",
                            h.old_start(),
                            h.old_lines(),
                            h.new_start(),
                            h.new_lines()
                        )
                    } else {
                        "@@ -0,0 +0,0 @@".to_string()
                    };
                    current_hunk = Some(Hunk {
                        header,
                        lines: Vec::new(),
                    });
                }
                '+' => {
                    let content = String::from_utf8_lossy(line.content()).to_string();
                    let diff_line = DiffLine {
                        origin: DiffLineOrigin::Addition,
                        old_lineno: None,
                        new_lineno: line.new_lineno(),
                        content,
                    };
                    deltas[idx].additions += 1;
                    if let Some(h) = current_hunk.as_mut() {
                        h.lines.push(diff_line);
                    }
                }
                '-' => {
                    let content = String::from_utf8_lossy(line.content()).to_string();
                    let diff_line = DiffLine {
                        origin: DiffLineOrigin::Deletion,
                        old_lineno: line.old_lineno(),
                        new_lineno: None,
                        content,
                    };
                    deltas[idx].deletions += 1;
                    if let Some(h) = current_hunk.as_mut() {
                        h.lines.push(diff_line);
                    }
                }
                ' ' => {
                    let content = String::from_utf8_lossy(line.content()).to_string();
                    let diff_line = DiffLine {
                        origin: DiffLineOrigin::Context,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                        content,
                    };
                    if let Some(h) = current_hunk.as_mut() {
                        h.lines.push(diff_line);
                    }
                }
                _ => {}
            }

            true
        })?;

        // Flush last hunk
        if let Some(h) = current_hunk.take() {
            if let Some(idx) = current_delta_idx {
                deltas[idx].hunks.push(h);
            }
        }

        Ok(deltas)
    }
}
