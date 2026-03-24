# Spec: Go-to-Line & Fuzzy File Picker (Issue #63)

**Priority**: P1 (UX Improvement — Owner-Filed)
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)
**Addresses**: GitHub Issue #63

## Problem

mdiff lacks two standard power-user navigation shortcuts that users expect from any vim-style TUI:

1. **`:<number>` go-to-line**: In the diff view, typing `:<number>` should jump to that line. This is a universal vim convention. Currently, `:` opens the settings menu, which conflicts.
2. **`Ctrl+P` fuzzy file picker**: A quick fuzzy-search modal to jump to any file in the diff by typing part of its name. This is the standard "quick open" pattern from VS Code, Sublime Text, and every modern editor.

The owner specifically requested these as default keybinding remaps to lower cognitive burden.

## Design

### 1. `:<number>` Go-to-Line

**Behavior**:
- When the user presses `:` in the diff view, open a command input bar at the bottom of the screen (similar to vim's command line)
- If the user types a number and presses Enter, scroll the diff view to that line number
- If the user types anything else (or just `:help`), open the settings/help menu as before
- Escape cancels the command input

**Implementation**:

The current `:` binding opens the settings menu directly. We need to intercept `:` to first show a command input bar, then dispatch based on input:

- **New state**: Add `command_input: Option<TextBuffer>` to `AppState` (or a `CommandBarState` struct with `active: bool`, `buffer: TextBuffer`)
- **New action**: `OpenCommandBar`, `CommandBarChar(char)`, `CommandBarBackspace`, `CommandBarConfirm`, `CommandBarCancel`
- **Event mapping**: `:` in diff view → `OpenCommandBar` (instead of `OpenSettings`)
- **Command parsing**: On confirm:
  - If buffer matches `^\d+$` → `GoToLine(n)` action — scroll diff to line `n`
  - If buffer == `help` → `OpenSettings` (or a help view)
  - Otherwise → show "Unknown command" in status bar
- **Rendering**: Show a single-line input bar at the bottom of the diff view with `:` prefix (like vim)

**Go-to-line logic**: The line number should refer to the **new file** line numbers shown in the diff gutter. Find the display row where `new_lineno == target` and scroll to it. If the line number is in a collapsed/hidden context section, expand it first or scroll to the nearest visible line.

**Files affected**:
- `src/state/app_state.rs` — Add `CommandBarState`
- `src/event.rs` — Map `:` to `OpenCommandBar`, handle command bar key events
- `src/action.rs` — Add `OpenCommandBar`, `CommandBarChar`, `CommandBarBackspace`, `CommandBarConfirm`, `CommandBarCancel`, `GoToLine(u32)`
- `src/app.rs` — Handle new actions, implement go-to-line scroll logic
- `src/components/` — Add command bar renderer (simple one-line bar at bottom)
- `src/components/which_key.rs` — Update help text for `:`

### 2. `Ctrl+P` Fuzzy File Picker

**Behavior**:
- `Ctrl+P` opens a centered modal with a text input at the top and a filtered list of files below
- As the user types, files are fuzzy-matched and the list updates in real-time
- Up/Down (or Ctrl+K/Ctrl+J) navigates the filtered list
- Enter selects the file and jumps to it in the diff view
- Escape closes the picker

**Implementation**:

This overlaps with the existing Command Palette spec (018) but is specifically scoped to file picking only, which is simpler and what the owner requested.

- **New state**: Add `FilePicker` struct with `active: bool`, `query: TextBuffer`, `filtered_entries: Vec<usize>` (indices into `diff.deltas`), `selected: usize`
- **Fuzzy matching**: Use the `nucleo` crate (already available in the project) for fuzzy matching file paths
- **New actions**: `OpenFilePicker`, `FilePickerChar(char)`, `FilePickerBackspace`, `FilePickerUp`, `FilePickerDown`, `FilePickerConfirm`, `FilePickerCancel`
- **Event mapping**: `Ctrl+P` → `OpenFilePicker`
- **On confirm**: Set `diff.selected_file` to the selected delta index and close the picker

**Rendering**:
- Centered floating modal (similar to existing modals like annotation menu)
- Top: text input with search icon/prefix
- Below: list of matched files with fuzzy match highlights
- Show file path and change stats (+N/-N) for each entry
- Highlight matched characters in the file path

**Files affected**:
- `src/state/app_state.rs` — Add `FilePickerState`
- `src/state/` — New `file_picker_state.rs` module
- `src/event.rs` — Map `Ctrl+P` to `OpenFilePicker`, handle picker key events
- `src/action.rs` — Add file picker actions
- `src/app.rs` — Handle picker actions, fuzzy matching logic
- `src/components/` — New `file_picker.rs` renderer
- `src/components/which_key.rs` — Add `Ctrl+P` to help

### Keybinding Conflict Resolution

- `:` currently opens settings. After this change, `:` opens the command bar. `:help` (or `:settings`) opens settings. This matches vim convention.
- `Ctrl+P` is currently unmapped. The Command Palette spec (018) also uses `Ctrl+P` — this file picker IS the first phase of the command palette. It can be extended later to include actions, not just files.

## Testing

1. In diff view, press `:42` Enter — should scroll to line 42 in the diff
2. Press `:` then Escape — should cancel without opening settings
3. Press `:help` Enter — should open settings/help
4. Press `:999999` Enter — should scroll to end of file or show "line not found"
5. Press `Ctrl+P` — should open file picker modal
6. Type part of a filename — list should filter in real-time with fuzzy matching
7. Arrow keys should navigate the filtered list
8. Enter should select the file and show its diff
9. Escape should close the picker
10. With 50+ files, picker should remain responsive
