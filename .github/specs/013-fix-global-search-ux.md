# Spec: Fix Global Search UX (Issue #35)

**Priority**: P0 (Regression Bug)
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (2-3 files changed)
**Addresses**: GitHub Issue #35

## Problem

The global search feature (Ctrl+F), merged in PR #13, has two critical UX bugs:

1. **`n` key conflict**: In the global search input handler (`event.rs` line 389), `KeyCode::Char('n')` is matched as `GlobalSearchNext` *before* the catch-all `KeyCode::Char(c) => GlobalSearchChar(c)` on line 391. This means the user literally cannot type the letter 'n' into the search query. Any search containing 'n' (e.g., "function", "return", "handler") is impossible.

2. **No-results ambiguity**: When the search bar is open with an empty query, the UI shows keybinding hints (`[n]next [N]prev [Esc]close`). When the user types a query that has no matches, it shows "No matches found" in red. But there is no visual state for "results exist, navigate them" vs "still typing" — the transition is jarring and the user doesn't know when to stop typing and start navigating.

## Root Cause

In `src/event.rs`, the global search active handler (Priority 2.8) maps keys in a single `match` block:

```rust
KeyCode::Char('n') => Some(Action::GlobalSearchNext),  // line 389
KeyCode::Char('N') => Some(Action::GlobalSearchPrev),   // line 390
KeyCode::Char(c) => Some(Action::GlobalSearchChar(c)),   // line 391
```

The `'n'` and `'N'` arms take precedence over the wildcard `c` arm, making it impossible to type these characters into the search query.

## Proposed Solution

### Fix 1: Use Enter/Ctrl+N for navigation instead of bare `n`/`N`

Change the global search key handling to use a two-phase approach:
- **While typing (search bar focused)**: ALL character keys insert into the query. No exceptions.
- **Navigation keys**: Use `Ctrl+N` / `Ctrl+P` (or `Enter` to confirm and begin navigating with `n`/`N` after closing the search bar) for next/prev match.

Specifically, update the key mapping in `src/event.rs` Priority 2.8 block:

```rust
if ctx.global_search_active {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => Some(Action::TextCursorHome),
            KeyCode::Char('e') => Some(Action::TextCursorEnd),
            KeyCode::Char('w') => Some(Action::TextDeleteWord),
            KeyCode::Char('n') => Some(Action::GlobalSearchNext),
            KeyCode::Char('p') => Some(Action::GlobalSearchPrev),
            _ => None,
        };
    }
    return match key.code {
        KeyCode::Esc => Some(Action::EndGlobalSearch),
        KeyCode::Enter => Some(Action::GlobalSearchNext),  // Enter navigates to next match
        KeyCode::Backspace => Some(Action::GlobalSearchBackspace),
        KeyCode::Left => Some(Action::TextCursorLeft),
        KeyCode::Right => Some(Action::TextCursorRight),
        KeyCode::Home => Some(Action::TextCursorHome),
        KeyCode::End => Some(Action::TextCursorEnd),
        KeyCode::Char(c) => Some(Action::GlobalSearchChar(c)),  // ALL chars insert
        _ => None,
    };
}
```

### Fix 2: Improve search result feedback in the UI

Update `src/components/global_search_bar.rs` to show three distinct states:

1. **Empty query**: Show keybinding hints with updated keys: `[Ctrl+N]next [Ctrl+P]prev [Esc]close`
2. **Query with matches**: Show match count and current match info (already works), plus updated nav hints
3. **Query with no matches**: Show "No matches found" (already works)

The keybinding hints in the empty-query state should be updated to reflect the new Ctrl+N/Ctrl+P navigation keys instead of bare n/N.

### Fix 3: Keep search matches visible after closing

When the user presses `Esc` to close the global search bar, the current match position should be preserved so they can continue navigating with the per-file `/` search or re-open global search to continue. Currently `EndGlobalSearch` sets `active = false` but preserves matches — verify this works correctly.

## Files to Modify

1. **`src/event.rs`**: Update Priority 2.8 global search key mapping — move `n`/`N` navigation to `Ctrl+N`/`Ctrl+P`, let bare chars always insert
2. **`src/components/global_search_bar.rs`**: Update keybinding hints to show `Ctrl+N`/`Ctrl+P` instead of `n`/`N`
3. **`src/components/which_key.rs`** (if applicable): Update which-key help text for global search bindings

## Testing

1. Open global search with `Ctrl+F`
2. Type "function" — all characters including 'n' should appear in the search input
3. Press `Ctrl+N` to navigate to next match
4. Press `Ctrl+P` to navigate to previous match
5. Press `Enter` to navigate to next match (alternative)
6. Press `Esc` to close search bar
7. Verify match position is preserved after closing

## Backward Compatibility

This changes the keybindings for global search navigation from `n`/`N` to `Ctrl+N`/`Ctrl+P`. Since the feature was broken (couldn't type 'n'), no users have built muscle memory around the old bindings.
