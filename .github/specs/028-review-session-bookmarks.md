# Spec: Review Session Bookmarks (Quick Line Markers)

**Priority**: P2 (Quick Win — Clear UX Benefit)
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (4-5 files changed)
**Inspiration**: Vim marks (`m` + letter), VS Code bookmarks extension, Telescope.nvim picker pattern

## Problem

During code review of large agent changesets, reviewers frequently encounter lines that are "interesting but not annotation-worthy" — code they want to come back to, patterns they want to cross-reference, or sections that need a second look after reviewing related files. Currently, the only way to mark a line is to create a full annotation (which requires entering a comment), or to mentally remember scroll positions.

This creates a friction gap between "I notice this" and "I have a specific comment about this." Bookmarks fill that gap with zero-effort line marking.

## Design

### Data Model

```rust
// src/state/bookmark_state.rs (new file)

#[derive(Debug, Clone)]
pub struct Bookmark {
    /// File path relative to repo root
    pub file_path: String,
    /// Line number in the new file (preferred) or old file
    pub line: u32,
    /// Whether this is a new-file or old-file line
    pub is_new_line: bool,
    /// Optional single-character label (a-z, like vim marks)
    pub label: Option<char>,
    /// Timestamp for ordering
    pub created_at: u64,
}

#[derive(Debug, Default)]
pub struct BookmarkState {
    /// All bookmarks in creation order
    pub bookmarks: Vec<Bookmark>,
    /// Current bookmark index for navigation (-1 = none selected)
    pub current_index: Option<usize>,
    /// Whether the bookmark list panel is visible
    pub list_visible: bool,
}
```

### Keybindings

| Key | Context | Action |
|-----|---------|--------|
| `b` | Diff View (normal mode) | Toggle bookmark on current line |
| `B` | Diff View (normal mode) | Toggle bookmark list panel |
| `]b` | Diff View (normal mode) | Jump to next bookmark (in current file, then across files) |
| `[b` | Diff View (normal mode) | Jump to previous bookmark |
| `'` + letter | Diff View (normal mode) | Jump to named bookmark (a-z) |
| `m` + letter | Diff View (normal mode) | Set named bookmark at current line |
| `d` | Bookmark list panel | Delete selected bookmark |
| `Enter` | Bookmark list panel | Jump to selected bookmark |

Note: `b` is currently unused in diff view normal mode. `m` is used in navigator for "mark reviewed" but is free in diff view.

### Actions

```rust
// Add to src/action.rs
ToggleBookmark,                    // b: toggle bookmark at cursor line
ToggleBookmarkList,                // B: show/hide bookmark list panel
NextBookmark,                      // ]b: jump to next bookmark
PrevBookmark,                      // [b: jump to previous bookmark
SetNamedBookmark(char),            // m + letter: set named bookmark
JumpToNamedBookmark(char),         // ' + letter: jump to named bookmark
BookmarkListUp,                    // k in bookmark list
BookmarkListDown,                  // j in bookmark list
BookmarkListSelect,                // Enter in bookmark list
BookmarkListDelete,                // d in bookmark list
```

### UI Elements

#### 1. Gutter Bookmark Indicator

In the diff view gutter (left of line numbers), show a bookmark marker:

```
  │ 42 │   fn process_input(data: &[u8]) -> Result<()> {
◆ │ 43 │       let parsed = parse(data)?;          ← bookmarked line (diamond marker)
  │ 44 │       validate(&parsed)?;
◆a│ 45 │       transform(&parsed)                   ← named bookmark 'a' 
  │ 46 │   }
```

- Unnamed bookmarks: `◆` (diamond) in the accent color
- Named bookmarks: `◆` + letter in the accent color

#### 2. Bookmark List Panel

When `B` is pressed, show a right-side panel (similar to annotation menu) listing all bookmarks:

```
┌─ Bookmarks (4) ──────────────────┐
│ ◆  src/api/handler.rs:43         │
│ ◆a src/api/handler.rs:45         │
│ ◆  src/db/queries.rs:120         │
│ ◆b tests/integration/api.rs:15   │
└──────────────────────────────────┘
```

Navigation with `j`/`k`, jump with `Enter`, delete with `d`.

#### 3. Status Bar Indicator

Show bookmark count in the status bar: `◆ 4 bookmarks`

### Integration Points

- **Session persistence** (`src/session.rs`): Save/restore bookmarks with the review session
- **Feedback export** (`src/export.rs`): Optionally include bookmarks as "points of interest" in exports
- **Navigator**: Show bookmark count per file in the file list (e.g., `handler.rs [◆2]`)
- **Annotations**: Offer quick action to "convert bookmark to annotation" — opens comment editor pre-filled with bookmark location

### Session Persistence

Add bookmark serialization to `src/session.rs`:

```rust
#[derive(Serialize, Deserialize)]
struct BookmarkEntry {
    file_path: String,
    line: u32,
    is_new_line: bool,
    label: Option<char>,
}
```

## Implementation Steps

1. Create `src/state/bookmark_state.rs` with `Bookmark`, `BookmarkState`
2. Add `bookmark: BookmarkState` to `AppState` in `src/state/app_state.rs`
3. Add bookmark actions to `src/action.rs` (`ToggleBookmark`, `NextBookmark`, etc.)
4. Implement bookmark action handlers in `src/app.rs`
5. Update `src/components/diff_view.rs` to render gutter bookmark markers
6. Add bookmark list panel component in `src/components/bookmark_list.rs`
7. Update `src/event.rs` with keybinding mappings
8. Update `src/components/which_key.rs` with bookmark entries
9. Add bookmark persistence to `src/session.rs`

## Edge Cases

- **Bookmark on deleted line**: Store as old-file line reference. Show with `(deleted)` label in bookmark list.
- **Duplicate bookmarks**: Toggle behavior — pressing `b` on an already-bookmarked line removes the bookmark.
- **Named bookmark conflict**: Re-assigning a letter (e.g., `ma` when `a` already exists) moves the bookmark to the new location (vim behavior).
- **File no longer in diff**: Bookmarks for files removed from the diff are preserved but shown as dimmed/inactive in the list.
- **Max bookmarks**: No hard limit, but bookmark list panel scrolls when > 20 bookmarks.

## User Benefit

Reduces the friction of "flag and continue" review workflow from ~5 keystrokes (enter visual mode, open comment editor, type something, confirm) to 1 keystroke (`b`). For a 50-file agent changeset, a reviewer might bookmark 15-20 interesting spots during a first pass, then revisit them systematically with `]b`/`[b` navigation. Named bookmarks (`m` + letter) enable cross-file reference patterns: "bookmark `a` is the function definition, bookmark `b` is where it's called — are they consistent?"
