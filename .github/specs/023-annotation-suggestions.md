# Spec: Annotation Suggestions (Inline Code Proposals)

**Priority**: P1 (High Impact — Agent Feedback Differentiator)
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)

## Problem

When reviewing coding agent output in mdiff, reviewers can leave free-text comments on lines of code. But comments alone are ambiguous — saying "this should use a HashMap instead of a Vec" forces the agent (or a human) to interpret the intent and guess the desired implementation. There is no way to propose a specific replacement: "replace lines 42-45 with this exact code."

This is the difference between *describing* a fix and *showing* the fix. GitHub's "suggested changes" feature demonstrated that inline code proposals dramatically improve feedback quality — reviewers are more specific, agents can auto-apply suggestions, and the feedback loop tightens. No TUI diff viewer currently offers this capability. Plannotator has it in its GUI, but no terminal tool does.

For mdiff's core mission of structured feedback for coding agents, this is critical: agents parse structured suggestions far more reliably than free-text descriptions of what should change.

## Competitive Reference

- **GitHub suggested changes**: ` ```suggestion ` blocks in PR review comments, one-click apply
- **Plannotator**: Code suggestion annotation type (comment/suggestion/concern) in GUI
- **Anthropic Code Review**: Confidence-scored findings with specific remediation guidance
- **GitLab**: Inline code suggestions with multi-line support and batch apply

## Proposed Solution

### New Annotation Type: `CodeSuggestion`

Extend the annotation system with a new variant that carries both a comment and proposed replacement code.

### Data Model Changes

In `src/state/annotation_state.rs`, add a suggestion field to `Annotation`:

```rust
/// A single annotation attached to a range of diff lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub anchor: LineAnchor,
    pub comment: String,
    pub created_at: String,
    #[serde(default = "default_category")]
    pub category: AnnotationCategory,
    #[serde(default = "default_severity")]
    pub severity: AnnotationSeverity,
    /// Optional replacement code for "suggestion" annotations.
    /// When present, this annotation proposes replacing the anchored lines
    /// with this code block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_code: Option<String>,
}
```

This is backward-compatible — existing annotations without `suggested_code` deserialize with `None`.

### New Actions

Add to `src/action.rs`:

```rust
// Annotation suggestions
OpenSuggestionEditor,       // Ctrl+S on a selection/line: open suggestion editor
ConfirmSuggestion,          // Enter (in suggestion editor): save the suggestion
CancelSuggestion,           // Esc: cancel suggestion editor
SuggestionChar(char),       // Character input in suggestion editor
SuggestionBackspace,        // Backspace in suggestion editor
SuggestionNewline,          // Enter for newline in suggestion body
SuggestionPaste(String),    // Paste in suggestion editor
SuggestionCursorLeft,       // Cursor movement in editor
SuggestionCursorRight,
SuggestionCursorUp,
SuggestionCursorDown,
ToggleSuggestionPreview,    // Tab in suggestion editor: toggle preview of old→new
```

### Workflow

1. **Select lines**: User navigates to a line or enters visual mode (`v`) to select a range — this is existing behavior.
2. **Open suggestion editor**: Press `Ctrl+S` (for "suggest"). This opens a two-pane editor at the bottom of the diff view:
   - **Left pane**: The original code (read-only, from the selected lines)
   - **Right pane**: Editable text area pre-populated with the original code
3. **Edit replacement**: User modifies the right pane to show the desired code. Standard text editing: type, backspace, paste, arrow keys.
4. **Add comment (optional)**: A single-line comment bar above the code panes for explaining *why* this change is suggested.
5. **Preview**: Press `Tab` to toggle a unified diff preview showing the old→new change inline.
6. **Confirm**: Press `Ctrl+Enter` to save the suggestion as an annotation with `suggested_code` populated and category auto-set to `Suggestion`.
7. **Cancel**: Press `Esc` to discard.

### New State Module

Create `src/state/suggestion_state.rs`:

```rust
use crate::state::text_buffer::TextBuffer;

#[derive(Debug, Default)]
pub struct SuggestionState {
    /// Whether the suggestion editor is currently open.
    pub active: bool,
    /// The original code being replaced (read-only display).
    pub original_code: Vec<String>,
    /// Editable buffer for the replacement code.
    pub replacement: TextBuffer,
    /// Optional comment explaining the suggestion.
    pub comment: TextBuffer,
    /// Whether the preview pane is visible.
    pub preview_visible: bool,
    /// Which pane is focused: Comment, Replacement
    pub focus: SuggestionFocus,
    /// File path and line ranges for the anchor.
    pub file_path: String,
    pub old_range: Option<(u32, u32)>,
    pub new_range: Option<(u32, u32)>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SuggestionFocus {
    Comment,
    #[default]
    Replacement,
}
```

Register in `src/state/mod.rs`:
```rust
pub mod suggestion_state;
pub use suggestion_state::SuggestionState;
```

Add to `AppState`:
```rust
pub suggestion: SuggestionState,
```

### New Component

Create `src/components/suggestion_editor.rs`:

**Layout** (when active, rendered as an overlay at bottom of diff view area):
```
┌─ Suggest replacement for lines 42-45 ─────────────────────────┐
│ Comment: [Use HashMap for O(1) lookup instead of Vec scan    ]│
├────────────────────────┬──────────────────────────────────────┤
│ Original (read-only)   │ Replacement (edit)                   │
│                        │                                      │
│ let found = items      │ let found = map                      │
│   .iter()              │   .get(&key)                         │
│   .find(|i| i.key     │   .cloned();                         │
│     == key)            │                                      │
│   .cloned();           │                                      │
│                        │                                      │
├────────────────────────┴──────────────────────────────────────┤
│ Ctrl+Enter: Save  Tab: Preview  Esc: Cancel                   │
└───────────────────────────────────────────────────────────────┘
```

**Preview mode** (toggle with Tab):
```
┌─ Preview ──────────────────────────────────────────────────────┐
│ - let found = items                                            │
│ -   .iter()                                                    │
│ -   .find(|i| i.key == key)                                    │
│ -   .cloned();                                                 │
│ + let found = map                                              │
│ +   .get(&key)                                                 │
│ +   .cloned();                                                 │
└───────────────────────────────────────────────────────────────┘
```

Register in `src/components/mod.rs`:
```rust
pub mod suggestion_editor;
```

### Rendering Suggestions in Diff View

When a suggestion annotation exists on a line range, modify `src/components/diff_view.rs` to show a visual indicator:

- Lines covered by a suggestion get a colored left-border marker (e.g., yellow `▐`)
- Below the anchored lines, render a collapsed suggestion block:
  ```
  ──── 💡 Suggestion: Use HashMap for O(1) lookup (expand: Enter) ────
  ```
- When the cursor is on this block and user presses Enter, expand to show the replacement inline

### Feedback Export Integration

The existing structured feedback export (`Ctrl+E`) and prompt generation should include suggestions. In the JSON export format:

```json
{
  "annotations": [
    {
      "file": "src/lib.rs",
      "lines": "42-45",
      "category": "Suggestion",
      "severity": "Major",
      "comment": "Use HashMap for O(1) lookup instead of Vec scan",
      "suggested_code": "let found = map\n  .get(&key)\n  .cloned();"
    }
  ]
}
```

In the LLM prompt export, suggestions render as:

```
## Suggestion (Major) — src/lib.rs:42-45
Use HashMap for O(1) lookup instead of Vec scan

Replace:
\`\`\`rust
let found = items
  .iter()
  .find(|i| i.key == key)
  .cloned();
\`\`\`

With:
\`\`\`rust
let found = map
  .get(&key)
  .cloned();
\`\`\`
```

### Key Mapping

In `src/event.rs`:

**Global binding** (Priority 4, when not in a text input mode):
```rust
// Ctrl+S: Open suggestion editor (when a line/selection is active in diff view)
KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
    return Some(Action::OpenSuggestionEditor);
}
```

**When suggestion editor is active** (new Priority 1 handler):
```rust
// Suggestion editor key handling
if state.suggestion.active {
    match key.code {
        KeyCode::Esc => return Some(Action::CancelSuggestion),
        KeyCode::Enter if modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::ConfirmSuggestion);
        }
        KeyCode::Enter => return Some(Action::SuggestionNewline),
        KeyCode::Tab => return Some(Action::ToggleSuggestionPreview),
        KeyCode::Backspace => return Some(Action::SuggestionBackspace),
        KeyCode::Char(c) => return Some(Action::SuggestionChar(c)),
        KeyCode::Left => return Some(Action::SuggestionCursorLeft),
        KeyCode::Right => return Some(Action::SuggestionCursorRight),
        KeyCode::Up => return Some(Action::SuggestionCursorUp),
        KeyCode::Down => return Some(Action::SuggestionCursorDown),
        _ => {}
    }
}
```

### Action Dispatch

In `src/app.rs`:

**`OpenSuggestionEditor`**:
1. Get current selection (from `SelectionState` or cursor position)
2. Extract the source code lines from the diff for the selected range
3. Populate `suggestion.original_code` with those lines
4. Pre-populate `suggestion.replacement` TextBuffer with the same lines
5. Set `suggestion.file_path`, `old_range`, `new_range` from the selection
6. Set `suggestion.active = true`

**`ConfirmSuggestion`**:
1. Build an `Annotation` with:
   - `anchor` from `suggestion.file_path`, `old_range`, `new_range`
   - `comment` from `suggestion.comment` buffer
   - `category: AnnotationCategory::Suggestion`
   - `severity: AnnotationSeverity::Major` (default, user can change after)
   - `suggested_code: Some(suggestion.replacement.text())`
2. Add to `AnnotationState`
3. Reset `SuggestionState` to default

**`CancelSuggestion`**:
1. Reset `SuggestionState` to default

### Which-Key Integration

Add to `src/components/which_key.rs`:
```
Ctrl+S  Suggest replacement code
```

When suggestion editor is active, show:
```
Ctrl+Enter  Save suggestion
Tab         Toggle preview
Esc         Cancel
```

## Files to Modify

1. **`src/state/annotation_state.rs`**: Add `suggested_code: Option<String>` to `Annotation`
2. **`src/state/suggestion_state.rs`** (NEW): `SuggestionState`, `SuggestionFocus`
3. **`src/state/mod.rs`**: Register `suggestion_state` module
4. **`src/action.rs`**: Add suggestion-related actions
5. **`src/event.rs`**: Add Ctrl+S binding, suggestion editor key handling
6. **`src/app.rs`**: Handle suggestion actions (open, confirm, cancel, text input)
7. **`src/components/suggestion_editor.rs`** (NEW): Render the two-pane suggestion editor
8. **`src/components/diff_view.rs`**: Render suggestion indicators on annotated lines
9. **`src/components/mod.rs`**: Register `suggestion_editor` module
10. **`src/components/which_key.rs`**: Add suggestion keybindings to help overlay
11. **`src/components/feedback_summary.rs`**: Include suggestions in summary/export

## Testing

1. Open mdiff on a diff with code changes
2. Navigate to a line, press `Ctrl+S` — suggestion editor opens with that line pre-populated
3. Enter visual mode (`v`), select a range, press `Ctrl+S` — editor shows the selected range
4. Modify the replacement code in the right pane
5. Type a comment in the comment bar
6. Press `Tab` — see unified diff preview of old→new
7. Press `Tab` again — back to editor view
8. Press `Ctrl+Enter` — suggestion saved, editor closes
9. In diff view, see the suggestion indicator on the annotated lines
10. Press `Ctrl+E` to export — verify suggestion appears with `suggested_code` in JSON
11. Open feedback summary — verify suggestion displays with replacement code
12. Press `Esc` during editing — editor closes, no annotation created
13. Run `cargo check` — no compilation errors
14. Run `cargo test` — existing tests pass

## Edge Cases

- Empty replacement: Allow it (represents "delete these lines")
- Single-line selection: Editor shows one line, works normally
- Very long lines: Horizontal scroll in both panes
- Terminal too narrow for side-by-side: Stack panes vertically
- Paste handling: Support `TextPaste` action in suggestion editor
- Existing annotation on same range: Allow multiple annotations (comment + suggestion)

## Backward Compatibility

- `suggested_code` field uses `#[serde(default, skip_serializing_if = "Option::is_none")]` so existing serialized annotations deserialize without issues
- `Ctrl+S` is currently unmapped
- No existing keybindings or behavior changes
- Purely additive feature
