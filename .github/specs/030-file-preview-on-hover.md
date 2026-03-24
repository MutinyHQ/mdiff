# Spec: File Preview on Hover in Navigator (Issue #65)

**Priority**: P0 (Bug Fix — Owner-Filed UX Regression)
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (2-3 files changed)
**Addresses**: GitHub Issue #65

## Problem

When hovering (moving the cursor over) a file in the file explorer/navigator, the diff pane should update to show a preview of the hovered file's diff. Currently, it continues to show the last *selected* file's diff. The issue is specific to tree view mode but should work consistently in both flat and tree views.

This breaks the expected "preview on hover" pattern that users rely on for quick scanning of changes across files in a large changeset. Without it, the user must explicitly select (Enter) each file to see its diff, adding friction to the review workflow.

## Design

### Current Behavior

In flat view, navigating up/down with `j`/`k` moves the cursor AND updates `diff.selected_file`, which triggers the diff view to re-render with the new file's content. In tree view, cursor movement updates the tree's highlighted entry but does NOT update `diff.selected_file` when the highlighted entry is a file (as opposed to a directory).

### Required Behavior

When the navigator cursor moves to a new entry (via `j`/`k`, arrow keys, or mouse hover):
1. If the entry is a **file** (not a directory): update `diff.selected_file` to the index of that file in `diff.deltas`, causing the diff pane to show that file's diff
2. If the entry is a **directory**: do NOT change `diff.selected_file` — keep showing the current file's diff
3. This should work in both flat view and tree view

### Implementation

**Key area**: The navigator's cursor movement handler in tree mode. When `NavigatorUp` / `NavigatorDown` actions fire in tree mode:

1. After updating the tree cursor position, check if the new position points to a file entry
2. If it's a file, resolve the file's index in `state.diff.deltas` and set `state.diff.selected_file = Some(index)`
3. The diff view already re-renders based on `selected_file`, so no changes needed there

**Mapping tree entry to delta index**: The tree view stores entries as `NavigatorEntry` variants. When the entry is a `File` variant, its path should be matchable against `deltas[i].path` to find the correct delta index.

**Files to modify**:
- `src/app.rs` — In the `NavigatorUp`/`NavigatorDown` action handlers (and any tree-specific nav actions), after updating the tree cursor, sync `diff.selected_file` with the currently highlighted file
- `src/state/navigator_state.rs` — May need a helper method like `current_file_delta_index(&self, deltas: &[FileDelta]) -> Option<usize>` that resolves the currently highlighted tree entry to a delta index

**Mouse handling**: If mouse clicks in the navigator also need fixing (related to Issue #64's mouse bug), ensure mouse-driven cursor changes also trigger the same `selected_file` sync.

### Edge Cases

- Cursor on a directory entry: keep previous `selected_file` unchanged
- Cursor on a file not in deltas (shouldn't happen if tree is built from deltas): no-op
- Empty diff (no deltas): `selected_file` stays `None`
- Rapid cursor movement: each movement should immediately update the preview (no debouncing needed since it's just an index change, not a git operation)

## Testing

1. Open a multi-file diff in tree view
2. Use `j`/`k` to move cursor between files — diff pane should update on each file hover
3. Move cursor to a directory — diff pane should keep showing the last file
4. Switch to flat view — verify same hover-preview behavior works
5. Use mouse to hover/click files in tree view — diff should update
6. Verify no performance regression with rapid cursor movement on large changesets
