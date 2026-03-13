# Spec: Diff Statistics Dashboard (`S` toggle)

**Priority**: P1 (High Impact)
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)
**Roadmap item**: #6 — Diff Statistics Dashboard

## Problem

When a coding agent modifies 30+ files across a large changeset, reviewers must scroll through every file in the navigator to understand the scope of changes. There is no way to quickly answer:

1. How many files were changed, added, or deleted?
2. Which files have the most churn (additions + deletions)?
3. What's the distribution of changes across file types (`.rs`, `.toml`, `.md`)?
4. Are the changes concentrated in a few files or spread evenly?

Without this overview, reviewers either waste time scanning the full list or jump into review without understanding the changeset shape — leading to missed files and poor prioritization.

## Competitive Reference

- **GitHub PR stats bar**: Shows total files changed, additions, deletions with green/red bar per file
- **`git diff --stat`**: Classic one-line-per-file summary with `+`/`-` counts and ASCII bar chart
- **lazygit**: Shows file status with color-coded indicators but no aggregate stats
- **VS Code Source Control**: Groups files by status (modified, added, deleted) with counts

## Proposed Solution

### New State: `StatsState`

Add `src/state/stats_state.rs`:

```rust
#[derive(Debug, Default)]
pub struct StatsState {
    pub open: bool,
    pub scroll_offset: usize,
    pub selected_index: usize,
}
```

### New Actions

Add to `src/action.rs`:

```rust
// Statistics dashboard
ToggleStatsDashboard,
StatsDashboardUp,
StatsDashboardDown,
StatsDashboardSelect,  // Jump to selected file in diff view
StatsDashboardSort,    // Cycle sort mode: by churn, by additions, by name
```

### Key Mapping

In `src/event.rs`, add to Priority 4 (global bindings):

```rust
KeyCode::Char('S') => return Some(Action::ToggleStatsDashboard),
```

When the dashboard is open, add a priority handler:

```rust
if ctx.stats_dashboard_open {
    return match key.code {
        KeyCode::Esc | KeyCode::Char('S') => Some(Action::ToggleStatsDashboard),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::StatsDashboardUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::StatsDashboardDown),
        KeyCode::Enter => Some(Action::StatsDashboardSelect),
        KeyCode::Char('s') => Some(Action::StatsDashboardSort),
        _ => None,
    };
}
```

### Computing Statistics

The statistics should be computed from the existing `DiffState` data. Create a function in the stats module:

```rust
pub struct FileStats {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub status: FileStatus,  // Added, Modified, Deleted, Renamed
}

pub struct DiffSummary {
    pub total_files: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub files_added: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
    pub files_renamed: usize,
    pub by_extension: Vec<(String, usize)>,  // (extension, file count)
    pub file_stats: Vec<FileStats>,          // sorted by churn descending
}

pub enum SortMode {
    ByChurn,       // additions + deletions, descending
    ByAdditions,   // additions only, descending
    ByName,        // alphabetical
}
```

The `DiffSummary` should be computed once when the diff is loaded and cached. It should be recomputed when the diff changes (e.g., after staging/unstaging files).

### UI Component

Create `src/components/stats_dashboard.rs`:

Render as a full-screen overlay (similar to feedback summary), with:

**Header section** (top 4-5 lines):
```
┌─ Diff Statistics ─────────────────────────────────────────┐
│                                                            │
│  Files: 34 changed (12 added, 19 modified, 3 deleted)     │
│  Lines: +1,247 -389  (net +858)                           │
│  Types: .rs (18)  .toml (3)  .md (5)  .css (4)  other (4)│
│                                                            │
```

**File list section** (scrollable, remaining space):
```
│  Sort: by churn ▾  [s] cycle sort                          │
│────────────────────────────────────────────────────────────│
│ ▸ src/components/diff_view.rs      +187  -45  ████████░░  │
│   src/app.rs                       +134  -23  ██████░░░░  │
│   src/event.rs                     +98   -67  ███████░░░  │
│   src/state/annotation_state.rs    +76   -12  ████░░░░░░  │
│   src/components/navigator.rs      +54   -8   ███░░░░░░░  │
│   Cargo.toml                       +12   -3   █░░░░░░░░░  │
│   ...                                                      │
│                                                            │
│ [j/k] navigate  [Enter] jump to file  [s] sort  [Esc] close│
└────────────────────────────────────────────────────────────┘
```

**Visual elements:**
- Sparkline/bar for each file proportional to its churn relative to the max
- Green for additions, red for deletions in the bar
- Added files marked with `[A]`, deleted with `[D]`, renamed with `[R→]`
- Selected file highlighted with accent color
- File type badges in the header use distinct colors

### Action Dispatch

When the user selects a file (`StatsDashboardSelect`):
1. Close the dashboard
2. Navigate to the selected file in the navigator (`SelectFile` action)
3. Focus the diff view (`FocusDiffView` action)

This provides a workflow: scan overview → identify high-churn file → jump directly to it.

### KeyContext Update

Add `stats_dashboard_open: bool` to `KeyContext` in `event.rs`, populated from `state.stats.open`.

## Files to Modify

1. **`src/state/stats_state.rs`** (NEW): `StatsState`, `DiffSummary`, `FileStats`, `SortMode`
2. **`src/state/mod.rs`**: Add `pub mod stats_state;`
3. **`src/state/app_state.rs`**: Add `stats: StatsState` field, add `diff_summary: Option<DiffSummary>` for cached stats
4. **`src/action.rs`**: Add stats dashboard actions
5. **`src/event.rs`**: Add `stats_dashboard_open` to `KeyContext`, add priority handler, add `S` trigger
6. **`src/components/stats_dashboard.rs`** (NEW): Full-screen overlay UI component
7. **`src/components/mod.rs`**: Register new component
8. **`src/app.rs`**: Handle stats actions, compute/cache `DiffSummary`, call component render

## Testing

1. Open mdiff on a multi-file diff
2. Press `S` — dashboard opens showing aggregate stats
3. Verify file counts match navigator file count
4. Verify additions/deletions match actual diff content
5. Press `s` — sort cycles through churn → additions → name
6. Press `j`/`k` or arrows — navigate file list
7. Press `Enter` on a file — dashboard closes, navigator selects that file, diff view shows it
8. Press `Esc` — dashboard closes
9. Run `cargo check` — no compilation errors

## Edge Cases

- Empty diff (no files): Show "No changes to display"
- Single file: Still show dashboard but file list has one entry
- Binary files: Show as "binary" with no line counts
- Renamed files: Show old → new path
- Very long file paths: Truncate with ellipsis from the left (show filename, truncate directory prefix)
- Terminal resize while open: Re-render with new dimensions

## Backward Compatibility

No existing keybindings are changed. `S` (uppercase) is currently unmapped in the global context. This is purely additive.
