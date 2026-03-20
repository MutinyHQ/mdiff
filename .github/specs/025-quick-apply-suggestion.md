# Spec: Quick Apply Suggestion (Write Suggestion to Working Tree)

**Priority**: P1 (High Impact — Closes the Review-Edit Loop)
**Status**: Ready for implementation
**Estimated effort**: Medium (4-6 files changed)
**Depends on**: Annotation Suggestions (spec 023)

## Problem

The Annotation Suggestions feature (023) lets reviewers propose specific replacement code for lines they're reviewing. But after creating a suggestion, applying it requires leaving mdiff, opening the file in an editor, finding the right lines, and manually pasting the replacement. This breaks flow state and adds friction to the most valuable part of the review cycle.

CodeRabbit's 1-click apply and GitHub's "commit suggestion" demonstrate that one-key apply dramatically tightens the feedback loop. For mdiff's mission of structured feedback for coding agents, the ability to apply suggestions directly from the review tool — without context switching — makes the review process not just advisory but actively corrective.

## Competitive Reference

- **GitHub "commit suggestion"**: One-click to commit a suggested change directly from a PR review comment
- **CodeRabbit**: 1-click commit to apply AI-generated fixes from review comments
- **GitLab**: Batch apply multiple suggestions in a single commit
- **VS Code Quick Fix**: `Ctrl+.` to apply suggested code fixes inline

## Proposed Solution

### New Actions

Add to `src/action.rs`:

```rust
// Quick Apply Suggestion
ApplySuggestion,         // Ctrl+A on a suggestion annotation: apply to working tree
ConfirmApplySuggestion,  // 'y' in confirmation dialog
CancelApplySuggestion,   // 'n' or Esc in confirmation dialog
UndoApplySuggestion,     // Ctrl+Z after apply: revert via git checkout
```

### Workflow

1. **Navigate to suggestion**: User navigates to a line with a suggestion annotation (indicated by the yellow left-border marker from spec 023).
2. **Trigger apply**: Press `Ctrl+A` (for "apply"). This shows a confirmation dialog:
   ```
   ┌─ Apply suggestion? ─────────────────────────────────────┐
   │                                                          │
   │  File: src/lib.rs                                        │
   │  Lines: 42-45                                            │
   │                                                          │
   │  Replace:                                                │
   │    let found = items.iter().find(|i| i.key == key);      │
   │                                                          │
   │  With:                                                   │
   │    let found = map.get(&key).cloned();                   │
   │                                                          │
   │  [y] Apply  [n] Cancel                                   │
   └──────────────────────────────────────────────────────────┘
   ```
3. **Confirm**: Press `y` to apply. mdiff writes the replacement to the working tree file.
4. **Auto-refresh**: The diff view refreshes automatically to reflect the change.
5. **Visual feedback**: The annotation is marked as "applied" with a green checkmark indicator, and a status bar message confirms: "Suggestion applied to src/lib.rs:42-45".
6. **Undo**: Press `Ctrl+Z` within the same session to revert via `git checkout -- <file>` for the specific lines (or the whole file if line-level checkout is not feasible).

### New State

Add to `src/state/` (extend existing annotation or suggestion state):

```rust
/// Tracks the apply confirmation dialog state
#[derive(Debug, Default)]
pub struct ApplyConfirmState {
    /// Whether the confirmation dialog is active
    pub active: bool,
    /// Index of the annotation being applied
    pub annotation_index: usize,
    /// File path being modified
    pub file_path: String,
    /// Line range being replaced
    pub line_start: u32,
    pub line_end: u32,
    /// The replacement code
    pub replacement: String,
    /// The original code (for undo)
    pub original: String,
}

/// Tracks recently applied suggestions for undo
#[derive(Debug, Clone)]
pub struct AppliedSuggestion {
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub original_code: String,
    pub applied_code: String,
    pub timestamp: std::time::Instant,
}
```

Add to `AppState`:
```rust
pub apply_confirm: ApplyConfirmState,
pub applied_suggestions: Vec<AppliedSuggestion>,
```

### File Writing Logic

When applying a suggestion, mdiff needs to:

1. Read the target file from the working tree (not the diff — the actual file on disk)
2. Map the diff line numbers to file line numbers:
   - For "new" side lines: use the new file line numbers directly
   - For "old" side lines: use the old file line numbers against the original file
3. Replace the target line range with the suggestion's `suggested_code`
4. Write the modified content back to disk
5. Trigger a diff refresh (`Action::RefreshDiff`)

```rust
fn apply_suggestion_to_file(
    file_path: &str,
    line_start: u32,  // 1-indexed
    line_end: u32,     // 1-indexed, inclusive
    replacement: &str,
) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(file_path)?;
    let mut lines: Vec<&str> = content.lines().collect();

    // Store original for undo
    let original = lines[(line_start as usize - 1)..=(line_end as usize - 1)]
        .join("\n");

    // Replace the target range
    let replacement_lines: Vec<&str> = replacement.lines().collect();
    let start = (line_start as usize) - 1;
    let end = line_end as usize; // exclusive for splice
    lines.splice(start..end, replacement_lines);

    let new_content = lines.join("\n") + "\n";
    std::fs::write(file_path, &new_content)?;

    Ok(original)
}
```

### Annotation State Update

After applying, update the annotation to mark it as applied:

Extend `Annotation` in `src/state/annotation_state.rs`:
```rust
/// Whether this suggestion has been applied to the working tree
#[serde(default)]
pub applied: bool,
```

### Visual Indicators

In `src/components/diff_view.rs`, when rendering a suggestion annotation that has `applied: true`:
- Change the left-border marker from yellow to green
- Show a checkmark icon: `applied` instead of the expand prompt
- The annotation block shows: `[applied] Suggestion: Use HashMap for O(1) lookup`

### Key Mapping

In `src/event.rs`:

**When in diff view, not in any editor/dialog** (Priority 3):
```rust
// Ctrl+A: Apply suggestion at current line
KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
    return Some(Action::ApplySuggestion);
}
```

**When apply confirmation dialog is active** (Priority 1):
```rust
if state.apply_confirm.active {
    match key.code {
        KeyCode::Char('y') => return Some(Action::ConfirmApplySuggestion),
        KeyCode::Char('n') | KeyCode::Esc => return Some(Action::CancelApplySuggestion),
        _ => {}
    }
}
```

**Undo** (global, Priority 4):
```rust
// Ctrl+Z: Undo last applied suggestion
KeyCode::Char('z') if modifiers.contains(KeyModifiers::CONTROL) => {
    return Some(Action::UndoApplySuggestion);
}
```

### Action Dispatch

In `src/app.rs`:

**`ApplySuggestion`**:
1. Find the suggestion annotation at or near the current cursor position
2. If no suggestion found, show status message "No suggestion at cursor"
3. If found, populate `ApplyConfirmState` with the annotation details
4. Set `apply_confirm.active = true`

**`ConfirmApplySuggestion`**:
1. Read `apply_confirm` state
2. Call `apply_suggestion_to_file()` with the file path, line range, and replacement
3. On success:
   - Push to `applied_suggestions` for undo history
   - Mark the annotation as `applied = true`
   - Show status bar message: "Suggestion applied to {file}:{start}-{end}"
   - Dispatch `RefreshDiff` to reload the diff
4. On error:
   - Show status bar error: "Failed to apply: {error}"
5. Reset `apply_confirm` to default

**`CancelApplySuggestion`**:
1. Reset `apply_confirm` to default

**`UndoApplySuggestion`**:
1. Pop the most recent entry from `applied_suggestions`
2. If empty, show "Nothing to undo"
3. Write the `original_code` back to the file at the same line range
4. Mark the annotation as `applied = false`
5. Dispatch `RefreshDiff`
6. Show status bar message: "Suggestion reverted in {file}"

### Which-Key Integration

Add to `src/components/which_key.rs`:
```
Ctrl+A  Apply suggestion to file
Ctrl+Z  Undo last applied suggestion
```

When confirmation dialog is active:
```
y  Apply
n  Cancel
```

### Feedback Export Integration

In the structured feedback export (JSON and prompt), include the applied status:

```json
{
  "annotations": [
    {
      "file": "src/lib.rs",
      "lines": "42-45",
      "category": "Suggestion",
      "severity": "Major",
      "comment": "Use HashMap for O(1) lookup",
      "suggested_code": "let found = map.get(&key).cloned();",
      "applied": true
    }
  ]
}
```

## Files to Modify

1. **`src/action.rs`**: Add `ApplySuggestion`, `ConfirmApplySuggestion`, `CancelApplySuggestion`, `UndoApplySuggestion`
2. **`src/state/annotation_state.rs`**: Add `applied: bool` field to `Annotation`
3. **`src/state/mod.rs`**: Add `ApplyConfirmState`, `AppliedSuggestion` to `AppState` (or create new module)
4. **`src/event.rs`**: Add `Ctrl+A` binding, confirmation dialog key handling, `Ctrl+Z` undo binding
5. **`src/app.rs`**: Handle apply actions, file writing logic, undo logic, diff refresh
6. **`src/components/diff_view.rs`**: Render applied suggestion indicators (green border, checkmark)
7. **`src/components/which_key.rs`**: Add apply/undo keybindings to help overlay
8. **`src/components/feedback_summary.rs`**: Include applied status in export

## Testing

1. Create a suggestion annotation on a line range (requires spec 023 to be implemented first)
2. Press `Ctrl+A` on the suggestion — confirmation dialog appears
3. Verify the dialog shows the correct file, line range, original and replacement code
4. Press `y` — suggestion is applied to the working tree file
5. Verify the file on disk has been modified with the replacement code
6. Verify the diff view refreshes to reflect the change
7. Verify the annotation shows as "applied" with green indicator
8. Press `Ctrl+Z` — the change is reverted
9. Verify the file on disk is restored to original
10. Verify the annotation shows as "not applied" again
11. Press `n` during confirmation — dialog closes, no change made
12. Press `Ctrl+A` on a line without a suggestion — status message "No suggestion at cursor"
13. Run `cargo check` — no compilation errors
14. Run `cargo test` — existing tests pass

## Edge Cases

- **File modified externally between review and apply**: Read the file fresh before applying; if the target lines don't match the expected original, show a conflict warning
- **Suggestion applied to deleted file**: Check file exists before applying
- **Multiple suggestions on overlapping lines**: Apply one at a time; after first apply, subsequent suggestions may have shifted line numbers — warn the user
- **Empty replacement (deletion)**: Remove the target lines entirely
- **Undo after file modified again**: Warn that undo may cause conflicts
- **Binary files**: Skip — suggestions should only exist on text files

## Backward Compatibility

- `applied` field uses `#[serde(default)]` so existing annotations deserialize with `false`
- `Ctrl+A` is currently unmapped
- `Ctrl+Z` is currently unmapped
- No existing keybindings or behavior changes
- This feature is a no-op until spec 023 (Annotation Suggestions) is implemented — the apply action requires `suggested_code` to be present on an annotation
