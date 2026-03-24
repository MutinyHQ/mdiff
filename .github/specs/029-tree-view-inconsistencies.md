# Spec: Tree File Viewer Inconsistencies (Issue #64)

**Priority**: P0 (Bug Fix — Owner-Filed Regression)
**Status**: Ready for implementation
**Estimated effort**: Medium (4-6 files changed)
**Addresses**: GitHub Issue #64

## Problem

The tree viewer mode introduced in PR #56 has several inconsistencies compared to the flat file list view. The repo owner reports:

1. The `m` keybinding does not work in tree view to set file modified/review states
2. Directory collapse should use `x` key (more intuitive, matches vim's close-fold convention)
3. Tree vs flat view preference is not persistent — it resets on restart and is not accessible from the `:` settings menu
4. Mouse clicks in tree view select the wrong file (off-by-one or indentation offset issue)
5. Tree view displays gitignored files that should be filtered out

These issues make the tree view feel like a second-class citizen compared to the flat view, undermining its utility for the core use case of navigating large agent changesets.

## Design

### 1. `m` Keybinding in Tree View

The `m` key already maps to `ToggleFileReviewStatus` in the flat navigator. The tree view's key handling in `map_key_to_action` needs to ensure this action fires when `tree_mode` is active and the selected entry is a file (not a directory).

**Files affected**: `src/event.rs` (ensure `m` maps correctly in tree context), `src/app.rs` (handle action for tree-selected file)

### 2. Remap Directory Collapse to `x`

Currently, `Enter` toggles collapse on directories and `T` toggles tree mode. The owner wants `x` to collapse/toggle directory nodes. This is more intuitive — `x` is a common "close" gesture in vim-style UIs.

**Implementation**:
- In `src/event.rs`, when the navigator is focused and tree mode is active: map `x` to `TreeToggleCollapse` (or a new action if the current one doesn't toggle per-node)
- Keep `Enter` for selecting/opening files, but on directories `Enter` should still toggle collapse as a secondary binding
- `x` should be the primary documented binding for collapse

**Files affected**: `src/event.rs`, `src/components/which_key.rs` (update help text)

### 3. Persistent Tree/Flat View Setting

The tree vs flat view toggle (`T`) should persist across sessions and be accessible in the `:` settings menu.

**Implementation**:
- Add `tree_mode: bool` to the settings/config state (likely in `src/state/settings_state.rs` or wherever persistent preferences are stored)
- Save the preference when toggled
- Load the preference on startup
- Add a "View Mode: Tree/Flat" entry in the `:` settings modal
- The `:` settings menu should be accessible via `:help` as the owner mentioned remapping

**Files affected**: `src/state/settings_state.rs`, `src/state/navigator_state.rs`, `src/components/settings_modal.rs`, `src/session.rs` (persistence)

### 4. Fix Mouse Click Target in Tree View

Mouse clicks in the tree view select the wrong file. This is likely because the mouse row calculation doesn't account for the tree's visual structure (directory headers, indentation, collapsed sections).

**Investigation needed**:
- Check `src/event.rs` or `src/components/navigator.rs` for mouse event handling
- The click row needs to be mapped through the tree's visible entries (accounting for collapsed directories and header rows)
- The flat view's mouse mapping is likely simpler (row = index), but the tree view needs to map through the rendered tree structure

**Files affected**: `src/event.rs` (mouse mapping), `src/components/navigator.rs` (tree rendering/click target calculation), `src/state/navigator_state.rs` (visible entry computation)

### 5. Filter Gitignored Files from Tree View

The tree view should respect `.gitignore` rules. Since mdiff already parses git diffs, it should only show files that are actually in the diff — gitignored files appearing suggests the tree construction is walking the filesystem rather than building from the diff's file list.

**Investigation needed**:
- Check how `NavigatorState` builds the tree entries in tree mode
- The tree should be built exclusively from `DiffState.deltas` (the files that have changes), not from a filesystem walk
- If directories are being added from the filesystem for tree structure, they should only include directories that contain changed files

**Files affected**: `src/state/navigator_state.rs` (tree construction logic)

## Keybinding Summary

| Key | Flat View | Tree View (after fix) |
|-----|-----------|----------------------|
| `m` | Toggle review status | Toggle review status (file only) |
| `x` | (unused) | Collapse/expand directory |
| `Enter` | Select file | Select file / toggle directory |
| `T` | Switch to tree | Switch to flat |
| `zM` | (n/a) | Collapse all |
| `zR` | (n/a) | Expand all |

## Testing

1. Open a diff with 10+ files across multiple directories
2. Switch to tree view with `T`
3. Verify `m` marks files as reviewed in tree view
4. Verify `x` collapses/expands directories
5. Verify mouse clicks select the correct file
6. Verify no gitignored files appear
7. Verify tree/flat preference persists after restart
8. Verify the setting appears in `:` menu
