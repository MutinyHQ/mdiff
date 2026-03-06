# Spec: Fix Diff Line Calculations (Issue #25)

**Priority**: P0 (Critical Bug)
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (2-4 files changed)
**Addresses**: GitHub Issue #25

## Problem

When using the `G` (ScrollToBottom) command in the diff view, the viewport does not render the full diff content. Lines exist below the viewport boundary that users cannot see, meaning the last portion of the diff is inaccessible. This is a critical usability bug — reviewers cannot see all changes, which directly undermines the core mission of thorough code review.

The issue is visible in the screenshot attached to #25: the diff view shows content cut off at the bottom with lines clearly existing beyond the rendered area.

## Root Cause Analysis

The bug likely stems from a mismatch between the **logical line count** (used for scroll bounds) and the **visual row count** (actual rendered rows after line wrapping and hunk headers). Key areas to investigate:

### 1. `ScrollToBottom` Action Handler (src/app.rs)

The `ScrollToBottom` action sets `state.diff.scroll_offset` based on some calculated maximum. If this maximum underestimates the total number of display rows, the viewport won't scroll far enough.

Look for code like:
```rust
Action::ScrollToBottom => {
    // The max scroll calculation may not account for:
    // - Hunk header lines (one per hunk)
    // - Line wrapping in split view
    // - Gap/expand-context placeholder lines
    // - The difference between content rows and viewport height
}
```

### 2. Display Map Row Count (src/display_map.rs)

The `build_display_map()` function computes the mapping from display rows to diff content. The total row count from this map must match what the scroll logic uses. If `build_display_map` returns N rows but the scroll bound uses a different calculation, the last rows become unreachable.

### 3. Split vs Unified View Differences

The bug may manifest differently (or only) in one view mode. Split view has additional complexity:
- Left and right panels may have different logical line counts
- Hunk headers span both panels
- Gap lines for expand-context add rows

### 4. Visual Row Metrics (src/components/diff_view.rs)

The `VisualRowMetrics` struct tracks `row_offsets`, `row_heights`, and `total_rows`. If line wrapping is enabled, a single logical row may occupy multiple visual rows. The scroll bound must use `total_rows` (visual) not the logical row count.

## Solution

### Step 1: Audit Scroll Bound Calculation

Find where `ScrollToBottom` sets the scroll offset. The correct formula is:

```rust
// total_display_rows = number of rows from build_display_map()
// viewport_height = inner area height (area.height - 2 for borders)
// max_scroll = total_display_rows.saturating_sub(viewport_height)
state.diff.scroll_offset = max_scroll;
```

If line wrapping is enabled, use `VisualRowMetrics::total_rows` instead of the display map length.

### Step 2: Ensure Consistent Row Counting

Verify that the same row-counting logic is used in:
1. `build_display_map()` — for content rendering
2. Scroll offset clamping — for navigation bounds
3. `cursor_row` bounds — for cursor movement limits

All three must agree on the total number of rows.

### Step 3: Handle Edge Cases

- **Empty hunks**: A hunk with only a header and no lines still occupies 1 display row
- **Expand-context gaps**: Gap placeholder lines count as display rows
- **Binary files**: Should not contribute to row count
- **Last line without newline**: Ensure the final line is counted

### Step 4: Add Scroll Clamping

After every scroll operation (not just `ScrollToBottom`), clamp the offset:

```rust
fn clamp_scroll(&mut self) {
    let max = self.total_display_rows().saturating_sub(self.viewport_height());
    self.state.diff.scroll_offset = self.state.diff.scroll_offset.min(max);
}
```

## Files to Modify

1. **src/app.rs** — Fix `ScrollToBottom` handler, add scroll clamping after all scroll actions
2. **src/components/diff_view.rs** — Verify `VisualRowMetrics` calculation is correct and exposed
3. **src/display_map.rs** — Audit `build_display_map()` total row count
4. **src/state/diff_state.rs** — May need to store `total_display_rows` for scroll bound reference

## Testing

- Manual test: Open a multi-hunk diff, press `G`, verify the last line of the last hunk is visible
- Manual test: Open a large diff (50+ hunks), press `G`, then `k` to scroll up one line — verify no content is hidden below
- Manual test: Test in both Split and Unified view modes
- Manual test: Test with line wrapping enabled (if applicable)
- Manual test: Test with expanded context (press `Space` to expand, then `G`)
- Unit test: Verify scroll bound calculation matches display map row count for various diff configurations

## Acceptance Criteria

- `G` scrolls to show the very last line of the diff
- No content exists below the viewport after `G`
- Works in both Split and Unified view modes
- Scroll offset is properly clamped after all navigation operations
- `j`/`k` navigation near the bottom does not leave hidden content
