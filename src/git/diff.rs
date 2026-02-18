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
                let commit = obj
                    .peel_to_commit()
                    .with_context(|| format!("{name} does not point to a commit"))?;
                let tree = commit.tree()?;
                repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))?
            }
            ComparisonTarget::Commit(oid) => {
                let commit = repo.find_commit(*oid)?;
                let tree = commit.tree()?;
                repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))?
            }
        };

        Self::parse_diff(&diff)
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
