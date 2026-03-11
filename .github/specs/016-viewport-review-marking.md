# Spec: Viewport-Based File Review Marking (Issue #43)

**Priority**: P0 (UX Friction)
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (2-3 files changed)
**Addresses**: GitHub Issue #43

## Problem

Currently, a file is only marked as "reviewed" when the user's cursor is on the last line of the diff. This forces users to manually navigate their cursor all the way to the bottom of every file, even when the entire diff is already visible in the viewport. For short files that fit within a single screen, the user can clearly see all changes but must still arrow-down to the last line to trigger the reviewed state. This adds unnecessary friction to the most common review workflow.

## Root Cause

The `ToggleFileReviewed` action (triggered by `w` key) and/or the implicit review detection logic only checks if `cursor_row` equals the last display row. It does not check whether the last display row is already visible within the current viewport (i.e., whether `scroll_offset + viewport_height >= total_display_rows`).

## Solution

### Approach: Auto-mark on viewport visibility

When the diff panel is focused and the last line of the current file's diff is visible within the viewport, automatically mark the file as reviewed. This should trigger on every scroll/navigation action that changes `scroll_offset` or `cursor_row`.

### File: `src/app.rs`

Add a check after any scroll or cursor movement action in the diff view. After processing `ScrollDown`, `ScrollToBottom`, `ScrollPageDown`, `JumpNextHunk`, or any navigation action that could reveal the last line:

```rust
// After processing scroll/cursor actions for the diff view:
fn check_viewport_review(&mut self) {
    // Only auto-mark when diff panel is focused
    if self.state.focus != FocusPanel::DiffView {
        return;
    }
    
    // Get total display rows for current file
    let total_rows = self.state.diff.visual_total_rows;
    if total_rows == 0 {
        return;
    }
    
    let viewport_bottom = self.state.diff.scroll_offset + self.state.diff.viewport_height;
    
    // If the last row is visible in the viewport, mark as reviewed
    if viewport_bottom >= total_rows {
        if let Some(delta_idx) = self.state.navigator.selected_delta_index() {
            if let Some(delta) = self.state.diff.deltas.get(delta_idx) {
                let path = delta.path.to_string_lossy().to_string();
                // Only mark if not already reviewed (don't toggle off)
                if self.state.review.status(&path) != FileReviewStatus::Reviewed {
                    self.state.review.mark_reviewed(&path);
                }
            }
        }
    }
}
```

Call `check_viewport_review()` after handling these actions:
- `ScrollDown`
- `ScrollToBottom`
- `ScrollPageDown`
- `JumpNextHunk`
- `SelectFile` (when a short file is selected and fits in viewport)
- `ScrollUp` / `ScrollPageUp` / `ScrollToTop` (user may scroll up after seeing the end)

Actually, the simplest approach is to call it after **every** action that modifies `scroll_offset`, `cursor_row`, or `selected_file` while the diff view is focused. This avoids missing edge cases.

### File: `src/state/review_state.rs`

No changes needed — `mark_reviewed()` already exists and is idempotent (calling it when already reviewed just updates the hash).

### File: `src/state/diff_state.rs`

Ensure `visual_total_rows` is always accurately computed after `build_display_map()` runs. This value is the key input to the viewport visibility check.

### Edge Cases

1. **File already reviewed**: `check_viewport_review` should be a no-op. Use `status()` check to avoid toggling off.
2. **Empty diff**: If `visual_total_rows == 0`, skip the check.
3. **Navigator focused**: Only trigger when `FocusPanel::DiffView` is active — the user should be looking at the diff, not the file list.
4. **File changes on refresh**: If diff content changes (hash mismatch), the `ReviewState::on_diff_refresh` method already handles marking it as `ChangedSinceReview`.
5. **Very large files**: The check is O(1) — just comparing integers, no performance concern.
6. **Manual toggle still works**: The `w` key `ToggleFileReviewed` action should continue to work as before, allowing users to manually mark/unmark files regardless of viewport position.

## Files Changed

- `src/app.rs` — Add `check_viewport_review()` helper, call it after scroll/navigation actions
- Possibly `src/state/review_state.rs` — Only if a `mark_reviewed_if_unreviewed()` convenience method is useful
- Possibly `src/components/diff_view.rs` — If `viewport_height` is set during rendering rather than in state

## Testing

1. `cargo check` passes
2. Open a diff with a short file (fits in one screen) — file should auto-mark as reviewed when selected and visible
3. Open a diff with a long file — scroll to the bottom, file should auto-mark when last line enters viewport
4. Press `w` to manually unmark a file — it should toggle to unreviewed
5. Scroll back up and then back down to the end — file should auto-mark again
6. Switch focus to navigator — scrolling should NOT trigger auto-mark
7. Refresh diff — changed files should show as `ChangedSinceReview`, not remain reviewed
