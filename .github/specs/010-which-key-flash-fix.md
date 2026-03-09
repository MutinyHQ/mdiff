# Spec: Fix Which-Key Dialog Flashing (Issue #27)

**Priority**: P0
**Status**: Ready for implementation
**Estimated effort**: Small (1-2 files changed)
**Addresses**: GitHub Issue #27

## Problem

Pressing `?` opens the which-key dialog, but it does not stay open — it flashes briefly while the key is held and disappears when released. This makes it impossible for users to read the available keybindings, completely defeating the purpose of the help overlay. The which-key feature was merged but is essentially broken for its core use case: discoverability.

## Root Cause Analysis

The `?` key is mapped to `Action::ToggleWhichKey` in `src/event.rs` (Priority 6 global bindings). The issue is that crossterm emits **both** a `KeyPress` and a `KeyRelease` event for the same key. When `?` is pressed:

1. `KeyPress` event fires → `ToggleWhichKey` → `which_key_visible = true`
2. `KeyRelease` event fires → `ToggleWhichKey` → `which_key_visible = false`

This causes the dialog to appear and immediately disappear, creating a "flash" effect.

**Alternatively**, the issue could be that the which-key state is being reset during tick processing or re-rendering. The `?` key in crossterm is actually `Shift+/`, which might cause the key repeat to rapidly toggle the state.

## Solution

### Approach: Filter KeyRelease events + make toggle idempotent

1. **In `src/event.rs` (EventReader)**: Filter out `KeyRelease` and `KeyRepeat` events at the event reader level. Crossterm's `EventStream` emits `KeyEventKind::Press`, `KeyEventKind::Repeat`, and `KeyEventKind::Release`. Only `KeyEventKind::Press` should be forwarded for most key handling.

2. **In the Event mapping**: Add a guard in `EventReader::new()` to only emit `Event::Key(key)` when `key.kind == KeyEventKind::Press`. This is the standard pattern used by ratatui example applications.

### Implementation Details

#### File: `src/event.rs`

In the `EventReader::new()` spawned task, change the Key event handling:

```rust
use crossterm::event::KeyEventKind;

// In the event reader task:
Some(Ok(CrosstermEvent::Key(key))) => {
    // Only forward key press events, not release/repeat
    if key.kind == KeyEventKind::Press {
        if event_tx.send(Event::Key(key)).is_err() {
            break;
        }
    }
}
```

This is a one-line change that fixes the root cause for ALL key events, not just `?`. It prevents any key from accidentally triggering twice due to press+release pairs.

#### Verification

After the fix:
- Press `?` → which-key dialog opens and stays open
- Press `?` again → dialog closes
- Press `Esc` → dialog closes (if we add Esc handling)
- No keys should trigger double actions from release events

### Additional Improvement (Optional)

Add `Esc` as an alternative way to close the which-key dialog. In `map_key_to_action`, add a check before the Priority 4 global bindings:

```rust
// Priority 3.75: Which-key overlay
if ctx.which_key_visible {
    return match key.code {
        KeyCode::Esc => Some(Action::ToggleWhichKey),
        KeyCode::Char('?') => Some(Action::ToggleWhichKey),
        _ => Some(Action::ToggleWhichKey), // Any key closes the overlay
    };
}
```

This would make the which-key overlay act as a modal that dismisses on any keypress, which is the expected UX pattern (similar to vim's which-key plugin).

## Files Changed

- `src/event.rs` — Filter `KeyEventKind::Press` only in EventReader (primary fix)
- `src/event.rs` — Add which-key modal dismiss logic in `map_key_to_action` (optional improvement)
- `src/state/app_state.rs` — No changes needed (which_key_visible already exists)

## Testing

1. Build with `cargo check`
2. Run mdiff on a repo with diffs
3. Press `?` — dialog should open and stay
4. Press any key — dialog should close
5. Verify no other keys have double-trigger behavior
