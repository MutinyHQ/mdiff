# 033 — Diff Timeline Scrubber

## Summary

Add a commit timeline scrubber that lets users navigate through individual commits within the diff range, viewing how changes evolved across agent iterations. Toggled with `Ctrl+T`, it shows a horizontal timeline bar at the bottom of the diff view with commit markers. Users scrub between commits using `<` and `>` keys, with the diff view updating to show only the changes from the selected commit.

## Motivation

Coding agents often produce multiple commits in a single session — iterating on code, fixing errors, adding tests. When reviewing the final diff, all these iterations are flattened into one view, making it impossible to understand the agent's reasoning process. vibetracer (released March 2026) validates this pattern with a timeline-based diff replay for AI edits.

mdiff already has attribution state (`src/state/attribution_state.rs`) that maps hunks to commits. The timeline scrubber extends this by allowing users to isolate individual commits and see the diff at each step, providing a "time travel" capability within the review workflow.

## User Stories

- **As a reviewer**, I want to see how the agent's changes evolved commit-by-commit so I can understand the reasoning behind each iteration.
- **As a reviewer**, I want to isolate a single commit's changes when a multi-commit changeset is overwhelming.
- **As a reviewer**, I want to scrub through the timeline quickly to find where a specific change was introduced.

## Design

### Architecture

1. **New state module**: `src/state/timeline_state.rs`
   - `TimelineState` struct:
     - `active: bool` — whether timeline mode is on
     - `commits: Vec<TimelineCommit>` — ordered list of commits in the range
     - `selected_index: Option<usize>` — None = show all (combined), Some(i) = show only commit i
     - `hover_index: Option<usize>` — for visual highlighting during navigation
   - `TimelineCommit` struct:
     - `oid: String` — commit SHA
     - `summary: String` — first line of commit message
     - `author: String`
     - `timestamp: String`
     - `files_changed: usize`
     - `additions: usize`
     - `deletions: usize`

2. **Commit collection**: Extend `src/git/attribution.rs`
   - Add `collect_timeline_commits()` function that returns `Vec<TimelineCommit>` with stats
   - Reuse existing `collect_commits()` logic but enrich with file change stats

3. **Diff filtering**: Extend `src/git/diff.rs`
   - Add method to compute diff for a single commit (parent..commit) instead of the full range
   - When timeline has a selected commit, swap the displayed deltas to show only that commit's changes
   - Cache the full combined diff so switching back to "all" is instant

4. **New component**: `src/components/timeline_bar.rs`
   - Horizontal bar rendered at the bottom of the diff view (above status bar)
   - Shows commit markers as colored dots/blocks on a timeline
   - Selected commit is highlighted with accent color
   - Shows commit summary, author, and stats for the hovered/selected commit
   - "ALL" marker at the left for combined view

5. **Actions** (in `src/action.rs`):
   - `Action::ToggleTimeline` — toggle timeline bar visibility
   - `Action::TimelineNext` — select next commit (right)
   - `Action::TimelinePrev` — select previous commit (left)
   - `Action::TimelineSelectAll` — return to combined view
   - `Action::TimelineSelect(usize)` — jump to specific commit

6. **Keybindings**:
   - `Ctrl+T` — toggle timeline bar
   - `>` / `.` — next commit in timeline
   - `<` / `,` — previous commit in timeline
   - `0` (when timeline active) — return to "all commits" combined view

### Integration Points

- **Attribution state**: Timeline reuses session data from `AttributionState`. When a commit is selected in the timeline, it's equivalent to filtering by that session in attribution.
- **Diff view title**: When a specific commit is selected, the diff title shows "[commit N/M] <summary>" to indicate filtered mode.
- **Navigator**: File list updates to show only files changed in the selected commit. When returning to "all", the full file list restores.
- **Review state**: Review progress tracks independently per timeline mode (combined vs per-commit).

### Data Flow

```
User presses Ctrl+T → ToggleTimeline action
  → App builds TimelineState from git history (reuse attribution commit collection)
  → Timeline bar renders at bottom of diff view
  → User presses > → TimelineNext
  → App computes single-commit diff (parent..selected_commit)
  → DiffState.deltas replaced with single-commit deltas
  → NavigatorState updated to show only affected files
  → User presses 0 → TimelineSelectAll
  → DiffState.deltas restored to full combined diff
```

## Acceptance Criteria

- [ ] `Ctrl+T` toggles the timeline bar at the bottom of the diff view
- [ ] Timeline shows commit markers with summaries
- [ ] `>` and `<` navigate between commits
- [ ] Selecting a commit filters the diff to show only that commit's changes
- [ ] `0` returns to the combined "all commits" view
- [ ] Navigator file list updates when commit is selected
- [ ] Diff view title indicates current commit when filtered
- [ ] Timeline works correctly with single-commit diffs (shows "1/1" state)
- [ ] `cargo check` passes
- [ ] `cargo test` passes (if tests exist)

## Files to Create/Modify

- **Create**: `src/state/timeline_state.rs`
- **Create**: `src/components/timeline_bar.rs`
- **Modify**: `src/action.rs` — add Timeline action variants
- **Modify**: `src/event.rs` — add key mappings (Ctrl+T, >, <, 0 in timeline context)
- **Modify**: `src/state/app_state.rs` — add `timeline: TimelineState` field
- **Modify**: `src/state/mod.rs` — re-export TimelineState
- **Modify**: `src/app.rs` — handle Timeline actions, compute per-commit diffs
- **Modify**: `src/git/diff.rs` — add single-commit diff computation
- **Modify**: `src/git/attribution.rs` — add `collect_timeline_commits()` with stats
- **Modify**: `src/components/which_key.rs` — add timeline context entries
