# Spec: Hunk-Level Navigation (`]` / `[` Keys)

**Priority**: P0
**Status**: Ready for implementation
**Estimated effort**: Small (2-3 files changed)

## Problem

mdiff currently supports line-by-line navigation (`j`/`k`) and page scrolling (`PageUp`/`PageDown`), but lacks the ability to jump directly between diff hunks — the fundamental unit of a diff. Every serious diff tool (vim-fugitive `[c`/`]c`, lazygit, tig, delta) supports hunk jumping. For large agent-generated diffs with hundreds of lines, reviewers need to skip context and jump between actual changes.

## Design

### Keybinding Changes

| Key | Current | New |
|-----|---------|-----|
| `]` | NextAnnotation | **JumpNextHunk** |
| `[` | PrevAnnotation | **JumpPrevHunk** |
| `Ctrl+]` | (unmapped) | NextAnnotation |
| `Ctrl+[` | (unmapped) | PrevAnnotation |

Rationale: Hunk navigation is needed far more frequently than annotation navigation. Annotations move to Ctrl-modified keys.

### Behavior

- `]` jumps cursor to the next hunk header (`is_header: true` in `DisplayRowInfo`)
- `[` jumps to the previous hunk header
- Wraps around: at the last hunk, `]` goes to the first hunk; at the first, `[` goes to the last
- Shows "Hunk N/M" in the status message bar
- Works in both split and unified view modes

## Implementation

### 1. `src/action.rs` — Add new action variants

```rust
// Add after ExpandContext:

// Hunk navigation
JumpNextHunk,
JumpPrevHunk,
```

### 2. `src/event.rs` — Update keybindings

**a.** In the "Annotation navigation (global in DiffExplorer)" section (~line 200), change bare `]`/`[` to hunk navigation:

```rust
// Annotation navigation moved to Ctrl modifier, hunk nav on bare keys
if ctx.active_view == ActiveView::DiffExplorer {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char(']') => return Some(Action::NextAnnotation),
            KeyCode::Char('[') => return Some(Action::PrevAnnotation),
            _ => {}
        }
    }
    match key.code {
        KeyCode::Char(']') => return Some(Action::JumpNextHunk),
        KeyCode::Char('[') => return Some(Action::JumpPrevHunk),
        _ => {}
    }
}
```

**Note:** The Ctrl+] and Ctrl+[ handling must come *before* the bare `]`/`[` check. Also verify these don't conflict with the Ctrl+C/Ctrl+D quit block above — they shouldn't since `]` and `[` are different chars.

### 3. `src/app.rs` — Add handler logic

Add two helper methods to the `App` impl. The exact field names depend on how the display map and cursor are stored — follow the patterns used by `ScrollUp`, `ScrollDown`, `ScrollToTop`, `ScrollToBottom`.

The key insight: `DisplayRowInfo` (from `src/display_map.rs`) has an `is_header: bool` field that is `true` for hunk boundary lines. The display map is a `Vec<DisplayRowInfo>` that maps display rows to diff data.

```rust
/// Find the display row index of the next hunk header after the current position.
fn find_next_hunk_row(&self, current_row: usize, display_map: &[DisplayRowInfo]) -> Option<usize> {
    // Search forward from current+1
    for idx in (current_row + 1)..display_map.len() {
        if display_map[idx].is_header {
            return Some(idx);
        }
    }
    // Wrap: search from beginning
    for idx in 0..=current_row {
        if display_map[idx].is_header {
            return Some(idx);
        }
    }
    None
}

/// Find the display row index of the previous hunk header before the current position.
fn find_prev_hunk_row(&self, current_row: usize, display_map: &[DisplayRowInfo]) -> Option<usize> {
    // Search backward from current-1
    if current_row > 0 {
        for idx in (0..current_row).rev() {
            if display_map[idx].is_header {
                return Some(idx);
            }
        }
    }
    // Wrap: search from end
    for idx in (0..display_map.len()).rev() {
        if display_map[idx].is_header {
            return Some(idx);
        }
    }
    None
}
```

In the action handler match block:

```rust
Action::JumpNextHunk => {
    // Get the display map and current cursor position following existing scroll patterns
    if let Some(row) = self.find_next_hunk_row(current_row, &display_map) {
        // Update scroll/cursor to `row` following the same pattern as ScrollToTop/ScrollToBottom
        // Show status
        let total_hunks = display_map.iter().filter(|r| r.is_header).count();
        let current_hunk = display_map[..=row].iter().filter(|r| r.is_header).count();
        self.state.status_message = Some((format!("Hunk {}/{}", current_hunk, total_hunks), false));
    }
}
Action::JumpPrevHunk => {
    if let Some(row) = self.find_prev_hunk_row(current_row, &display_map) {
        // Same pattern as above
        let total_hunks = display_map.iter().filter(|r| r.is_header).count();
        let current_hunk = display_map[..=row].iter().filter(|r| r.is_header).count();
        self.state.status_message = Some((format!("Hunk {}/{}", current_hunk, total_hunks), false));
    }
}
```

### 4. `README.md` — Update keybindings table

In the Navigation section, add:
```
| `]` | Jump to next hunk |
| `[` | Jump to previous hunk |
```

In the Annotations section, update:
```
| `Ctrl+]` / `Ctrl+[` | Jump to next/previous annotation |
```

## Testing

- Open mdiff on a repo with multiple hunks in a single file
- Verify `]` jumps to the next hunk header
- Verify `[` jumps to the previous hunk header
- Verify wrap-around works (last hunk → first, first → last)
- Verify "Hunk N/M" appears in the status bar
- Verify `Ctrl+]` and `Ctrl+[` still navigate annotations
- Test in both split and unified view modes
