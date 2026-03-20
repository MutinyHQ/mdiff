# Spec: Open-in-Editor at Line (`e` key)

**Priority**: P1 (High Impact — Universal Power-User Feature)
**Status**: Ready for implementation
**Estimated effort**: Small (3-4 files changed)

## Problem

When a reviewer spots an issue in a diff and wants to fix it immediately, they must:
1. Note the file path and line number
2. Exit mdiff (or open another terminal)
3. Open the file in their editor: `vim +42 src/lib.rs`
4. Navigate to the correct line

This context switch breaks review flow and is error-prone (wrong line number, wrong file). Every major TUI diff viewer competitor now supports jumping to the editor directly:
- **critique**: Click line number to open in editor (via `REACT_EDITOR` env var)
- **difi**: Press `e` to open file at exact line in vim/neovim
- **deff**: Press `e` to open in `$EDITOR`
- **lazygit**: Press `e` to edit file in `$EDITOR`

mdiff is missing this fundamental bridge between reviewing and editing.

## Competitive Reference

- **lazygit**: `e` key opens file in `$EDITOR` at the selected line
- **difi**: `e` key with `+line` argument for vim/neovim
- **deff**: `e` key, respects `$EDITOR` and `$VISUAL`
- **critique**: `REACT_EDITOR` env var for click-to-open
- **VS Code**: Click line number in diff view to jump to source

## Proposed Solution

### Keybinding

Press `e` when the diff view is focused to open the current file at the current line in the user's preferred editor.

### Editor Resolution

Resolve the editor command in this priority order (matching standard Unix convention):
1. `$VISUAL` environment variable (GUI editors)
2. `$EDITOR` environment variable (terminal editors)
3. Fallback to `vi`

### Line Number Detection

From the current scroll position and cursor in the diff view, determine:
- **File path**: From the currently selected file in the navigator (`state.navigator.selected_entry().path`)
- **Line number**: From the diff line at the current scroll offset, prefer the new-file line number. If on a deleted line (no new-file number), use the old-file line number.

The line number should map to the actual source file line, not the diff display line.

### Editor Invocation

Most editors support a `+LINE` argument for opening at a specific line:
- `vim +42 src/lib.rs`
- `nvim +42 src/lib.rs`
- `nano +42 src/lib.rs`
- `emacs +42 src/lib.rs`
- `code --goto src/lib.rs:42` (VS Code)
- `subl src/lib.rs:42` (Sublime Text)

For maximum compatibility, detect the editor name and use the appropriate format:

```rust
fn build_editor_command(editor: &str, file: &str, line: u32) -> std::process::Command {
    let editor_name = std::path::Path::new(editor)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(editor);

    let mut cmd = std::process::Command::new(editor);

    match editor_name {
        "code" | "code-insiders" => {
            cmd.arg("--goto").arg(format!("{}:{}", file, line));
        }
        "subl" | "sublime_text" => {
            cmd.arg(format!("{}:{}", file, line));
        }
        // vim, nvim, nano, emacs, vi, micro, helix, etc.
        _ => {
            cmd.arg(format!("+{}", line)).arg(file);
        }
    }

    cmd
}
```

### Terminal Handling

For terminal-based editors (vim, nvim, nano, emacs -nw, helix, micro):
1. **Suspend the TUI**: Disable raw mode, restore terminal state
2. **Spawn the editor**: Run as a child process, wait for exit
3. **Resume the TUI**: Re-enable raw mode, redraw

For GUI editors (code, subl):
1. **Spawn detached**: Don't wait for exit (editor opens in separate window)
2. **Keep TUI running**: No suspension needed

Detect terminal vs GUI by editor name:
```rust
fn is_gui_editor(editor_name: &str) -> bool {
    matches!(editor_name, "code" | "code-insiders" | "subl" | "sublime_text" | "atom" | "zed")
}
```

### File Path Resolution

The file path from the diff is relative to the git repository root. Resolve it to an absolute path:
```rust
let repo_root = state.repo_path(); // existing method to get repo root
let absolute_path = repo_root.join(&entry.path);
```

If the file is in a worktree, use the worktree root instead.

### New Action

Add to `src/action.rs`:

```rust
// Open in editor
OpenInEditor,
```

### Key Mapping

In `src/event.rs`, add to the diff view focus section (Priority 3 — diff view keybindings):

```rust
KeyCode::Char('e') => return Some(Action::OpenInEditor),
```

Note: `e` is currently unmapped in the diff view context. In visual mode, `e` is not used either (visual mode uses `v`, `j`, `k`, `Esc`).

### Action Handler

In `src/app.rs`, add a new helper module or function:

```rust
fn handle_open_in_editor(&mut self) -> anyhow::Result<()> {
    // 1. Get current file path
    let entry = self.state.navigator.selected_entry()
        .ok_or_else(|| anyhow::anyhow!("No file selected"))?;
    let file_path = entry.path.clone();

    // 2. Get current line number from diff view scroll position
    let line = self.state.diff.current_source_line()
        .unwrap_or(1);

    // 3. Resolve absolute path
    let abs_path = self.repo_root.join(&file_path);

    // 4. Resolve editor
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let editor_name = std::path::Path::new(&editor)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&editor)
        .to_string();

    // 5. Build command
    let mut cmd = build_editor_command(&editor, abs_path.to_str().unwrap(), line);

    if is_gui_editor(&editor_name) {
        // GUI editor: spawn detached, don't wait
        cmd.spawn()?;
    } else {
        // Terminal editor: suspend TUI, run editor, resume
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

        let status = cmd.status()?;

        crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        // Force full redraw
        self.dispatch(Action::Resize)?;

        if !status.success() {
            // Optionally show a status bar message
        }
    }

    // 6. Refresh diff in case the file was edited
    self.dispatch(Action::RefreshDiff)?;

    Ok(())
}
```

### DiffState Extension

Add a helper to `src/state/diff_state.rs` to map the current scroll position to a source line number:

```rust
impl DiffState {
    /// Get the source file line number at the current scroll position.
    /// Prefers new-file line number, falls back to old-file line number.
    pub fn current_source_line(&self) -> Option<u32> {
        // Use the current scroll offset to find the diff line
        // at the top of the viewport (or cursor position if implemented)
        let display_line = self.scroll_offset;
        // Look up the corresponding source line from parsed hunks
        self.source_line_at_display(display_line)
    }

    /// Map a display line index to a source file line number.
    fn source_line_at_display(&self, display_idx: usize) -> Option<u32> {
        // Iterate through parsed diff lines to find the one at display_idx
        // Return new_lineno if available, else old_lineno
        // Implementation depends on existing diff line data structures
        todo!("implement based on existing diff line parsing")
    }
}
```

### Which-Key Integration

Add to `src/components/which_key.rs` in the diff view section:

```
e       Open file in $EDITOR at current line
```

### Status Bar Feedback

After opening the editor (especially for GUI editors where the user stays in mdiff), briefly show in the context bar:
```
Opened src/lib.rs:42 in nvim
```

## Files to Modify

1. **`src/action.rs`**: Add `OpenInEditor` action
2. **`src/event.rs`**: Add `e` keybinding in diff view context
3. **`src/app.rs`**: Handle `OpenInEditor` — editor resolution, terminal suspend/resume, spawn
4. **`src/state/diff_state.rs`**: Add `current_source_line()` helper for line number mapping
5. **`src/components/which_key.rs`**: Add `e` to help overlay

## Testing

1. Set `$EDITOR=vim`, open mdiff on a diff
2. Navigate to a specific line in the diff view
3. Press `e` — mdiff suspends, vim opens the file at the correct line
4. Edit and save in vim, quit vim
5. mdiff resumes, diff refreshes to show any changes
6. Set `$EDITOR=code`, repeat — VS Code opens the file at the correct line, mdiff stays running
7. Unset both `$VISUAL` and `$EDITOR` — defaults to `vi`
8. Press `e` on a deleted line — opens at the old-file line number
9. Press `e` on an added line — opens at the new-file line number
10. Press `e` on a context line — opens at the correct line
11. Run `cargo check` — no compilation errors
12. Run `cargo test` — existing tests pass

## Edge Cases

- Editor not found in PATH: Show error in status bar, don't crash
- File deleted (only in old version): Show "file no longer exists" message
- Binary file: Show "cannot open binary file in editor" message
- No file selected: No-op
- Editor exits with error: Log but don't crash, resume TUI normally
- `$EDITOR` contains arguments (e.g., `"vim -u NONE"`): Split on spaces for the command
- Worktree files: Resolve path relative to the active worktree root

## Backward Compatibility

- `e` is currently unmapped in diff view context
- No existing keybindings or behavior changes
- Purely additive feature
