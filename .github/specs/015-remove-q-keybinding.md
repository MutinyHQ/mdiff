# Spec: Remove `q` Keybinding for Quit (Issue #38)

**Priority**: P0 (Data Loss Risk)
**Status**: Ready for implementation
**Estimated effort**: Small (2-3 files changed)
**Addresses**: GitHub Issue #38

## Problem

Pressing `q` immediately quits mdiff without any confirmation, even when there are staged annotations, line scores, checklist progress, or uncommitted review work. This is dangerous because:

1. A reviewer spends 20 minutes annotating a large agent changeset, accidentally hits `q`, and loses all their work
2. The `q` key is adjacent to `w` (whitespace toggle), `a` (annotation menu), and `s` (stage file) — common review actions
3. Unlike `Ctrl+C` and `Ctrl+D`, which require a double-press confirmation, `q` bypasses the safety net entirely

The issue reporter (repo owner) explicitly states: "just use ctrl-c or ctrl-d for exit."

## Root Cause

In `src/event.rs`, the global bindings (Priority 4) map `q` directly to `Action::Quit`:

```rust
KeyCode::Char('q') if !ctx.visual_mode_active => return Some(Action::Quit),
```

And in `src/app.rs`, `Action::Quit` sets `should_quit = true` immediately without any confirmation check:

```rust
Action::Quit => {
    self.state.should_quit = true;
}
```

Meanwhile, `Ctrl+C` and `Ctrl+D` go through `Action::ConfirmQuitSignal` which requires a double-press within a countdown window — a much safer pattern.

## Proposed Solution

### Option A (Recommended): Remove `q` entirely

Simply remove the `q` -> `Quit` mapping from `src/event.rs`. Users quit with `Ctrl+C` or `Ctrl+D` (double-press), which already has confirmation logic.

In `src/event.rs`, Priority 4, remove:
```rust
// DELETE THIS LINE:
KeyCode::Char('q') if !ctx.visual_mode_active => return Some(Action::Quit),
```

Also update:
1. **`src/components/which_key.rs`**: Remove `q` from the help overlay's quit hint. Update to show only `Ctrl+C` / `Ctrl+D` for quitting.
2. **`src/components/action_hud.rs`** (if applicable): Update any HUD text that mentions `q` for quit.

### Option B (Alternative): Route `q` through confirmation

If removing `q` entirely feels too aggressive, route it through the same confirmation flow as Ctrl+C/Ctrl+D:

```rust
KeyCode::Char('q') if !ctx.visual_mode_active => {
    return Some(Action::ConfirmQuitSignal(QuitCombo::CtrlC))
}
```

This would require pressing `q` twice to quit, matching the safety of Ctrl+C. However, Option A is cleaner and matches the issue reporter's explicit request.

### Additional Safety: Reclaim `q` for a useful action

With `q` freed up, it becomes available for a useful command. Potential uses:
- **No-op** (safest, recommended for now)
- Future: Quick-annotate shortcut, or cycle through annotation categories

For this spec, `q` should simply do nothing (be unmapped).

## Files to Modify

1. **`src/event.rs`**: Remove `KeyCode::Char('q')` -> `Action::Quit` mapping from Priority 4 global bindings
2. **`src/components/which_key.rs`**: Update help overlay to remove `q` from quit hint, show `Ctrl+C`/`Ctrl+D` only
3. **`src/action.rs`**: Optionally remove `Action::Quit` entirely if no other paths trigger it (but keep it if there are other quit paths)

**Important**: Do NOT remove `Action::Quit` from `action.rs` if it's still used by other code paths. Only remove the `q` key mapping in event.rs. The `Quit` action is still triggered by the double-press `ConfirmQuitSignal` flow.

## Testing

1. Open mdiff with a diff that has annotations/scores
2. Press `q` — nothing should happen
3. Press `Ctrl+C` once — see "Press Ctrl+C again to quit" message
4. Press `Ctrl+C` again — app quits
5. Press `Ctrl+D` twice — app quits
6. Verify `q` still works in contexts where it was never mapped (it wasn't mapped in visual mode, search mode, etc.)
7. Open which-key (`?`) — verify quit hint shows `Ctrl+C`/`Ctrl+D` only

## Backward Compatibility

This removes a keybinding that users may have muscle memory for. However:
- The issue was filed by the repo owner explicitly requesting this change
- The safety benefit (preventing accidental data loss) outweighs the convenience cost
- `Ctrl+C` and `Ctrl+D` are universal terminal quit shortcuts that all users know
- The which-key overlay will guide users to the correct quit method
