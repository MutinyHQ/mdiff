# Spec: File Tree Navigator with Collapsible Directory Hierarchy (Issue #53)

**Priority**: P1 (High Impact)
**Status**: Ready for implementation
**Estimated effort**: Medium-Large (6-8 files changed)
**Addresses**: GitHub Issue #53

## Problem

When a coding agent modifies 30+ files across multiple directories, the flat file list in the navigator becomes unwieldy. Reviewers cannot quickly see the structural scope of changes — "12 files changed in `src/components/`" vs "2 files changed in `src/state/`". They must scroll through every entry, mentally grouping files by directory. This slows down triage and makes it easy to miss entire directories of changes.

## Competitive Reference

- **VS Code file explorer**: Collapsible tree with indent guides, file counts per directory
- **GitHub PR file tree**: Sidebar tree grouping files by directory with expand/collapse
- **lazygit**: File tree view with directory grouping and aggregate stats
- **Yazi**: Rust TUI file manager with fast tree navigation, indent guides

## Proposed Solution

### Toggle Behavior

Add a `T` keybinding (already proposed in the issue) that toggles the navigator between:
1. **Flat mode** (current behavior): A flat list of files with abbreviated paths
2. **Tree mode** (new): Files grouped under collapsible directory nodes with indent guides

### New State: `TreeNode` and Navigator Extensions

Extend `src/state/navigator_state.rs` with tree data structures:

```rust
#[derive(Debug, Clone)]
pub enum TreeNodeKind {
    Directory {
        name: String,
        collapsed: bool,
        file_count: usize,
        total_additions: usize,
        total_deletions: usize,
    },
    File {
        entry_index: usize, // index into self.entries
    },
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub kind: TreeNodeKind,
    pub depth: usize,       // indentation level
    pub path: String,       // full path for directories, file path for files
}

// Add to NavigatorState:
pub tree_mode: bool,
pub tree_nodes: Vec<TreeNode>,
pub tree_selected: usize,
```

### Building the Tree

When `tree_mode` is toggled on (or when deltas update while in tree mode), build `tree_nodes` from the existing `entries`:

1. Parse all file paths into a trie/tree structure
2. For each directory that contains changed files, create a `Directory` node
3. Sort directories alphabetically, files alphabetically within each directory
4. Flatten the tree into `tree_nodes` Vec, respecting collapsed state
5. Compute aggregate stats (file_count, additions, deletions) per directory by summing children

### Visible Entries in Tree Mode

Add a method `visible_tree_nodes(&self) -> Vec<&TreeNode>` that returns only nodes that are visible (i.e., not inside a collapsed parent). This replaces `visible_entries()` when `tree_mode` is active.

### New Actions

Add to `src/action.rs`:

```rust
// Tree navigator
ToggleTreeView,
TreeToggleCollapse,    // Enter/l on a directory: toggle collapse
TreeCollapseAll,       // zM: collapse all directories
TreeExpandAll,         // zR: expand all directories
```

### Key Mapping

In `src/event.rs`, add to Priority 4 (global bindings):

```rust
KeyCode::Char('T') => return Some(Action::ToggleTreeView),
```

When tree_mode is active and focus is on navigator, intercept:

```rust
// In the navigator focus section, when tree_mode is active:
KeyCode::Enter | KeyCode::Char('l') => {
    // If selected node is a directory, toggle collapse
    // If selected node is a file, select the file (existing behavior)
    return Some(Action::TreeToggleCollapse);
}
KeyCode::Char('h') => {
    // If selected node is a file, collapse its parent directory
    // If selected node is a directory, collapse it
    return Some(Action::TreeToggleCollapse);
}
```

Add vim-style fold commands (only when tree_mode and navigator focused):

```rust
// After 'z' prefix detection:
KeyCode::Char('M') => return Some(Action::TreeCollapseAll),
KeyCode::Char('R') => return Some(Action::TreeExpandAll),
```

### UI Rendering

Modify `src/components/navigator.rs` to render tree nodes when `state.navigator.tree_mode` is true:

**Directory nodes:**
```
▶ src/components/ (8 files, +245 -67)
```
When expanded:
```
▼ src/components/
│ ├── ○ navigator.rs [M] +54 -8
│ ├── ✓ diff_view.rs [M] +187 -45
│ └── ★ stats_dashboard.rs [A] +45 -0
```

**Visual elements:**
- `▶` / `▼` for collapsed/expanded directories
- `│`, `├──`, `└──` indent guides using box-drawing characters
- Directory names in bold or accent color
- Aggregate stats `(N files, +X -Y)` in muted color for directories
- Review status icons (`✓`, `○`, `●`, `★`) still displayed per file
- Selected item highlighted with accent color and selection background

**Directory summary in collapsed state:**
When collapsed, show review progress inline: `(3/8 reviewed)`

### Action Dispatch

`TreeToggleCollapse`:
1. Get the selected tree node
2. If it's a Directory: toggle its `collapsed` field, rebuild `tree_nodes` flat list
3. If it's a File: dispatch `SelectFile(entry_index)` to open the diff view (same as current Enter behavior)

`TreeCollapseAll` / `TreeExpandAll`:
1. Set all Directory nodes to collapsed/expanded
2. Rebuild the flat `tree_nodes` list
3. Clamp selection

`ToggleTreeView`:
1. Toggle `state.navigator.tree_mode`
2. If switching to tree mode: build tree from current entries, set `tree_selected = 0`
3. If switching to flat mode: map current tree selection back to the flat list index

### Search in Tree Mode

The existing navigator search (`/`) should still work in tree mode:
- Filter by full file path (matching against `entry.path`)
- When search is active, temporarily flatten the tree (show only matching files, no directory nodes)
- When search is cancelled, restore the tree view with previous collapse state

### KeyContext Update

Add `tree_mode: bool` to `KeyContext` in `event.rs`, populated from `state.navigator.tree_mode`.

## Files to Modify

1. **`src/state/navigator_state.rs`**: Add `TreeNode`, `TreeNodeKind`, tree fields, tree building logic, `visible_tree_nodes()`
2. **`src/action.rs`**: Add `ToggleTreeView`, `TreeToggleCollapse`, `TreeCollapseAll`, `TreeExpandAll`
3. **`src/event.rs`**: Add `tree_mode` to `KeyContext`, add `T` global binding, modify navigator key handling for tree-specific actions
4. **`src/components/navigator.rs`**: Dual render path — flat mode (existing) and tree mode (new), indent guides, directory node rendering
5. **`src/app.rs`**: Handle new tree actions, rebuild tree on delta updates when tree_mode is active
6. **`src/components/which_key.rs`**: Add tree-mode keybindings to the help overlay

## Testing

1. Open mdiff on a multi-directory diff
2. Press `T` — navigator switches to tree view with directory grouping
3. Verify directories show aggregate stats (file count, +/-)
4. Press `Enter` on a directory — it collapses, hiding children
5. Press `Enter` again — it expands
6. Press `j`/`k` — navigation skips collapsed children
7. Press `Enter` on a file — opens diff view for that file
8. Press `zM` — all directories collapse
9. Press `zR` — all directories expand
10. Type `/` to search — tree flattens to show matching files
11. Press `Esc` to cancel search — tree restores
12. Press `T` — back to flat mode, selection preserved
13. Run `cargo check` — no compilation errors
14. Run `cargo test` — existing tests pass

## Edge Cases

- Single file diff: Tree mode shows just the file, no directories
- All files in same directory: One directory node with all files
- Deeply nested paths: Show full path with indent guides, truncate with ellipsis if needed
- Empty directory after filtering: Don't show empty directory nodes
- Terminal resize: Re-render with new dimensions

## Backward Compatibility

No existing keybindings are changed. `T` (uppercase) is currently unmapped. This is purely additive. Flat mode remains the default.
