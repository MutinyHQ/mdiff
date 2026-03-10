# Spec: Fix Paste in Comment Modal (Issue #36)

**Priority**: P0 (Workflow Blocker)
**Status**: Ready for implementation
**Estimated effort**: Small (1-2 files changed)
**Addresses**: GitHub Issue #36

## Problem

When the comment input modal is open and the user presses the system paste shortcut (Cmd+V on macOS, Ctrl+V on Linux), nothing happens. The pasted text is silently discarded. This forces users to manually type all annotation comments, which is especially painful for longer feedback or when copying error messages, code snippets, or links.

## Root Cause

In `src/app.rs`, the event processing loop (around line 364-365) handles paste events:

```rust
Event::Paste(text) if self.state.pty_focus => Some(Action::PtyPaste(text)),
Event::Paste(_) => None,
```

The `Event::Paste(_) => None` arm discards ALL paste events when the PTY is not focused. This means paste is completely broken in:
- Comment editor modal
- Commit message dialog
- Target input dialog
- Search inputs (navigator search, diff search, global search)
- Any other text input

The terminal emulator sends paste content as a `CrosstermEvent::Paste` event (when bracketed paste mode is enabled), but the app only routes it to the PTY runner and drops it everywhere else.

## Proposed Solution

### Step 1: Add a generic `TextPaste(String)` action

Add a new action variant to `src/action.rs`:

```rust
// Generic text input
TextPaste(String),
```

### Step 2: Route paste events based on active text input

In `src/app.rs`, replace the blanket discard with context-aware routing:

```rust
Event::Paste(text) if self.state.pty_focus => Some(Action::PtyPaste(text)),
Event::Paste(text) => Some(Action::TextPaste(text)),
```

### Step 3: Handle `TextPaste` in the action handler

In the action processing section of `src/app.rs`, add handling for `TextPaste` that routes to the currently active text input:

```rust
Action::TextPaste(text) => {
    if self.state.comment_editor_open {
        // Insert each character into the comment editor
        for c in text.chars() {
            if c == '\n' {
                self.state.comment_editor_text.insert_char('\n');
            } else if !c.is_control() {
                self.state.comment_editor_text.insert_char(c);
            }
        }
    } else if self.state.commit_dialog_open {
        for c in text.chars() {
            if c == '\n' {
                self.state.commit_message.insert_char('\n');
            } else if !c.is_control() {
                self.state.commit_message.insert_char(c);
            }
        }
    } else if self.state.target_dialog_open {
        for c in text.chars().filter(|c| !c.is_control()) {
            self.state.target_input.insert_char(c);
        }
    } else if self.state.navigator.search_active {
        for c in text.chars().filter(|c| !c.is_control()) {
            self.state.navigator.search_query.insert_char(c);
        }
        // Trigger search recomputation
        self.recompute_navigator_filter();
    } else if self.state.diff.search_active {
        for c in text.chars().filter(|c| !c.is_control()) {
            self.state.diff.search_query.insert_char(c);
        }
        self.recompute_diff_search_matches();
    } else if self.state.global_search.active {
        for c in text.chars().filter(|c| !c.is_control()) {
            self.state.global_search.query.insert_char(c);
        }
        self.recompute_global_search_matches();
    } else if self.state.agent_selector.open {
        for c in text.chars().filter(|c| !c.is_control()) {
            self.state.agent_selector.filter_query.insert_char(c);
        }
    }
    // If no text input is active, ignore the paste
}
```

**Note**: The exact field names and method names should be verified by reading the actual state structs. The pattern is: check which text input is active (in priority order matching event.rs), then insert each non-control character from the paste text into the appropriate `TextBuffer`.

### Step 4: Ensure bracketed paste mode is enabled

Crossterm's `EventStream` should already handle bracketed paste if terminal support is enabled. Verify that the terminal setup in `src/tui.rs` enables `PushKeyboardEnhancementFlags` or that bracketed paste is enabled via crossterm's terminal setup. If not, add:

```rust
crossterm::execute!(stdout, crossterm::event::EnableBracketedPaste)?;
```

to the terminal init, and the corresponding `DisableBracketedPaste` to cleanup.

## Files to Modify

1. **`src/action.rs`**: Add `TextPaste(String)` action variant
2. **`src/app.rs`**: Replace `Event::Paste(_) => None` with `Event::Paste(text) => Some(Action::TextPaste(text))`, and add `TextPaste` handler
3. **`src/tui.rs`** (potentially): Ensure bracketed paste mode is enabled

## Testing

1. Open comment editor (`i` on a diff line or `v` -> select -> `i`)
2. Copy text to clipboard externally
3. Press Cmd+V (macOS) or Ctrl+Shift+V (Linux terminal)
4. Verify pasted text appears in the comment editor
5. Repeat for commit dialog (`c`), search (`/`), global search (`Ctrl+F`), and target dialog (`t`)
6. Verify multi-line paste works in comment editor (newlines preserved)
7. Verify control characters are filtered out
8. Verify PTY paste still works when in PTY focus mode

## Backward Compatibility

No breaking changes. This purely adds functionality that was missing.
