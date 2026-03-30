# Spec: Fix Diff Scroll Bottom & Boundary Padding (Issue #25, Second Pass)

**Priority**: P0 (Critical Bug — Still Broken)
**Status**: Ready for implementation
**Estimated effort**: Small (3-5 files changed)
**Addresses**: GitHub Issue #25

## Problem

Despite PR #50 (which added `clamp_scroll()`), users still report that the bottom of the diff is cut off when pressing `G` (ScrollToBottom). Two separate follow-up comments on Issue #25 confirm:

1. "This is still an issue even after 0.1.14. I'm missing the bottom 3 lines from the diff."
2. "Still not seeing the bottom of the diff on long files that exceed the viewport. The last several lines are cut off."

Both users also request **boundary padding lines** — visible markers at the start and end of each file's diff that make it unambiguous whether the diff content is complete or truncated. E.g.:

```
~~~ Diff start for src/app.rs ~~~
... diff content ...
~~~ Diff end for src/app.rs ~~~
```

This is a two-part fix: (1) correct the scroll calculation that still clips bottom lines, and (2) add visual boundary markers.

## Root Cause (Scroll Clipping)

The `clamp_scroll()` fix from PR #50 likely under-counts the total display rows because it does not account for all of:
- The status bar / help bar at the bottom of the viewport (1 row)
- Hunk headers (each hunk has a decorative header row like `@@ -10,5 +10,7 @@`)
- The diff view's block border (top and bottom borders consume 2 rows)
- Possible expand-context gap placeholder rows
- The new boundary padding lines themselves (once added)

The fix should ensure `max_scroll = total_display_rows.saturating_sub(viewport_inner_height)` where `viewport_inner_height` is the actual number of renderable content rows inside the diff view block (after subtracting borders, title bar, etc.).

## Solution

### Part 1: Fix Scroll Clipping

1. In `src/app.rs`, find the `clamp_scroll()` function or scroll clamping logic.
2. Audit how `total_display_rows` is computed — it must match the count from `build_display_map()` exactly.
3. Audit how `viewport_height` is computed — it must subtract:
   - Block borders (2 rows for top+bottom border of the diff view `Block`)
   - Any title/header row inside the block
   - Any status/help row at the bottom
4. Store the viewport inner height after rendering layout so it's available to scroll calculations. If the current code uses `self.last_diff_rect` (or similar), make sure it calls `.inner()` on the Block to get the usable area.
5. Add a +1 or +2 padding to `total_display_rows` to ensure the very last line can be scrolled fully into view (the user should see the last line at the bottom of the viewport, not cut off at the edge).

### Part 2: Boundary Padding Lines

Add visual boundary markers to the display map so they appear as rendered rows in the diff:

1. In `src/display_map.rs`, modify `build_display_map()` to insert:
   - A **start boundary row** before the first hunk of each file: styled as dim/muted text, content like `─── diff start: {filename} ───`
   - An **end boundary row** after the last hunk of each file: styled as dim/muted text, content like `─── diff end: {filename} ───`

2. Add a new variant to `DisplayRowInfo` (or a flag):
   ```rust
   pub enum DisplayRowKind {
       HunkHeader,
       DiffLine,
       GapPlaceholder,
       BoundaryStart,  // NEW
       BoundaryEnd,    // NEW
   }
   ```

3. In `src/components/diff_view.rs`, render boundary rows with a distinctive dim style:
   - Use `theme.text_muted` color
   - Center the text or left-align with `~~~` delimiters
   - No line numbers in the gutter for boundary rows (show empty gutter)

4. Since boundary rows are now part of the display map, they are automatically counted in `total_display_rows`, fixing the off-by-N issue for scroll calculations.

### Part 3: Scroll Safety Net

After all scroll operations (ScrollDown, ScrollUp, ScrollToBottom, ScrollToTop, PageDown, PageUp, hunk navigation, etc.), call `clamp_scroll()` to ensure bounds are respected. Currently it may only be called in some code paths.

Add a universal clamp at the end of the action dispatch loop:
```rust
// At the end of the main action match block:
self.clamp_scroll();
```

This ensures no scroll action can leave the viewport in an invalid state.

## Files to Modify

1. **`src/display_map.rs`**: Add `BoundaryStart`/`BoundaryEnd` row kinds, insert boundary rows in `build_display_map()`
2. **`src/components/diff_view.rs`**: Render boundary rows with dim muted style, empty gutter
3. **`src/app.rs`**: Fix `clamp_scroll()` viewport height calculation, ensure clamp is called after all scroll actions
4. **`src/state/diff_state.rs`**: If needed, add field to cache `viewport_inner_height` for scroll calculations

## Testing

1. Open a diff with 20+ hunks in a single file. Press `G`. Verify the `─── diff end ───` boundary line is visible at the bottom.
2. Press `g` to go to top. Verify the `─── diff start ───` boundary line is visible at the top.
3. Press `G` then `k` — the line above the end boundary should be visible. No content hidden below.
4. Test in both Split and Unified view modes.
5. Test with a very short diff (1-2 lines) — boundary lines should still appear, scroll should not break.
6. Test with expanded context (`Space` to expand) then `G` — still shows full bottom.
7. Run `cargo check` — no compilation errors.
8. Run `cargo test` — existing tests pass.

## Acceptance Criteria

- `G` always shows the very last displayable row (the end boundary)
- Boundary padding lines appear at the start and end of each file's diff
- Boundary lines are visually distinct (dim/muted text, no line numbers)
- No content is ever hidden below the viewport after pressing `G`
- Works in both Split and Unified view modes
- Scroll clamping is applied after every scroll-related action
