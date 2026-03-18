# Spec: Review Progress Bar

**Priority**: P1 (High Impact — promoted from P2)
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (4-5 files changed)
**Roadmap item**: Review Progress Bar

## Problem

When reviewing a large coding agent changeset (30+ files), there is no persistent visual indicator of how far along the review is. The reviewer must mentally track which files they have reviewed by scanning the navigator's per-file review icons (✓/○). This creates two failure modes:

1. **Missed files**: Without a clear "X of Y reviewed" signal, reviewers may think they are done when unreviewed files remain, especially when files are filtered or scrolled out of view.
2. **No sense of progress**: Large reviews feel endless because there is no progress feedback. Reviewers lose motivation without a visible completion percentage advancing as they work.

The existing `ReviewState` already tracks per-file review status and exposes `reviewed_count()`. The navigator already renders per-file review icons. But there is no **aggregate** progress indicator — the information exists in the data model but is not surfaced in the UI.

## Competitive Reference

- **GitHub PR**: Shows "X/Y files viewed" with a progress bar at the top of the files changed tab
- **Gerrit**: Shows review completion percentage in the change header
- **lazygit**: No explicit progress bar, but staged/unstaged counts provide implicit progress
- **critique (TUI)**: No review progress tracking
- **VS Code Source Control**: Shows counts by status but no review progress

mdiff has review state tracking that no competitor TUI offers — surfacing it with a progress bar would be a clear differentiator.

## Proposed Solution

### UI: Progress Bar in Navigator Title/Footer

Render a compact progress bar in the navigator panel's bottom border (title_bottom), replacing or augmenting the current scroll position indicator (`1/34`). The progress bar shows:

```
┌─ Files (34) ──────────────────────┐
│ ▶ ✓ s/components/diff_view.rs ... │
│   ○ s/app.rs [M] +134 -23        │
│   ✓ s/event.rs [M] +98 -67       │
│   ...                             │
│                                   │
└─ ██████████░░░░░░ 18/34 (53%) ───┘
```

The bar uses block characters (`█` for reviewed, `░` for unreviewed) and shows the count and percentage. Color-coded: green (`theme.success`) for the filled portion, muted for unfilled.

### New State: No new state module needed

The existing `ReviewState` already has everything needed:
- `reviewed_count()` returns the number of reviewed files
- `status(path)` returns per-file status
- The navigator's `entries` provides the total file count

We only need a helper method to compute the progress fraction.

### New Method on ReviewState

Add to `src/state/review_state.rs`:

```rust
/// Returns (reviewed, total) for progress display.
pub fn progress(&self, total_files: usize) -> (usize, usize) {
    (self.reviewed_count(), total_files)
}
```

### Navigator Component Changes

Modify `src/components/navigator.rs` to render the progress bar in the bottom border:

```rust
// After computing scroll_info, build progress bar
let total_files = state.navigator.entries.len();
let reviewed = state.review.reviewed_count();
let progress_pct = if total_files > 0 {
    (reviewed as f64 / total_files as f64 * 100.0) as u8
} else {
    0
};

// Build the bar: width adapts to available space
let bar_max_width = (area.width as usize).saturating_sub(20); // leave room for "18/34 (53%)"
let filled = if total_files > 0 {
    (bar_max_width * reviewed) / total_files
} else {
    0
};
let unfilled = bar_max_width.saturating_sub(filled);

let bar_filled = "█".repeat(filled);
let bar_unfilled = "░".repeat(unfilled);
let progress_text = format!(" {bar_filled}{bar_unfilled} {reviewed}/{total_files} ({progress_pct}%) ");

// Use title_bottom with styled spans
let progress_line = Line::from(vec![
    Span::styled(
        format!(" {bar_filled}"),
        Style::default().fg(theme.success),
    ),
    Span::styled(
        format!("{bar_unfilled} "),
        Style::default().fg(theme.text_muted),
    ),
    Span::styled(
        format!("{reviewed}/{total_files} ({progress_pct}%) "),
        Style::default().fg(if reviewed == total_files && total_files > 0 {
            theme.success
        } else {
            theme.text
        }),
    ),
]);
```

The existing `scroll_info` in title_bottom should be replaced with this progress line. The scroll position (`1/34`) is already partially redundant with the progress bar's count and can be merged: show scroll position on the left and progress on the right, or replace entirely.

### Layout Decision: Replace vs Augment Scroll Info

Replace the current bottom title with a combined format:

```
└─ 3/34 ── ████████░░░░ 18/34 reviewed ─┘
```

Left side: current scroll position (`selected + 1 / total`).
Right side: review progress bar with count.

This preserves both pieces of information without requiring extra vertical space.

### Key Mapping

No new keybindings needed. The progress bar updates automatically as the user toggles file review status with the existing `ToggleFileReviewed` action (mapped to `r` in navigator context).

### Completion Celebration

When `reviewed == total_files` and `total_files > 0`, apply a visual flourish:
- Change the progress bar color to green (`theme.success`)
- Change the count text to bold green
- Optionally show a brief status message: "Review complete!"

This provides satisfying positive feedback for completing a review.

## Files to Modify

1. **`src/state/review_state.rs`**: Add `progress()` helper method
2. **`src/components/navigator.rs`**: Render progress bar in bottom border, replace/augment scroll_info
3. **`src/theme.rs`** (if needed): Ensure `success` color is available (likely already is — used for ✓ icon)

## Testing

1. Open mdiff on a multi-file diff (10+ files)
2. Verify progress bar appears in navigator bottom border showing `0/N (0%)`
3. Navigate to a file and press `r` to mark reviewed
4. Verify progress bar updates to `1/N (X%)`
5. Mark several files reviewed — verify bar fills proportionally
6. Mark all files reviewed — verify completion state (green, bold)
7. Unmark a file (`r` again) — verify bar decreases
8. Use file search (`/`) to filter files — verify progress still reflects total files, not filtered count
9. Resize terminal — verify bar adapts to available width
10. Run `cargo check` — no compilation errors
11. Run `cargo test` — existing tests pass

## Edge Cases

- **Zero files**: Show no progress bar (or `0/0`)
- **Single file**: Show `0/1 (0%)` or `1/1 (100%)`
- **Very narrow terminal**: Omit the bar portion, show only `X/Y` count
- **Terminal width < 20 chars**: Show only count, no bar
- **Files added/removed on refresh**: Progress resets for changed files (existing `ChangedSinceReview` logic handles this)
- **All files new after refresh**: Progress reflects new file count correctly

## Backward Compatibility

No existing keybindings or UI elements are removed. The scroll position indicator in the navigator footer is augmented, not replaced. The `reviewed_count()` method already exists on `ReviewState`. This is purely additive UI work.

## Future Extensions

- **Per-directory progress**: When File Tree Navigator (spec 020) lands, show progress per directory node
- **Progress in Diff Statistics Dashboard**: Show review progress alongside file stats (spec 019)
- **Progress persistence**: Save review state to disk so progress survives restarts (currently in-memory only)
