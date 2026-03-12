# Spec: Command Palette (`Ctrl+P`)

**Priority**: P1 (High Impact)
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)
**Roadmap item**: New — Command Palette for action discoverability

## Problem

mdiff has grown to 50+ distinct actions (see `src/action.rs`) across navigation, annotations, search, git operations, agent management, review state, and settings. The which-key overlay (`?`) shows keybindings for the current context, but users must:

1. Know which panel they're in to see relevant bindings
2. Scroll through a static list to find the action they want
3. Memorize keybindings for infrequent actions (export feedback, toggle checklist, open settings)

As the feature set grows, discoverability becomes a bottleneck. New users don't know what's available, and experienced users forget bindings for rarely-used features.

## Competitive Reference

- **VS Code** (`Ctrl+Shift+P`): The gold standard — fuzzy-searchable list of all commands with keybinding hints
- **Fresh editor** (`Ctrl+P`): New Rust terminal editor with command palette as a core UX pattern
- **Neovim Telescope**: Fuzzy finder for commands, files, and more — keyboard-driven with preview
- **lazygit**: No command palette, but context menus serve a similar purpose

## Proposed Solution

### New State: `CommandPaletteState`

Add `src/state/command_palette_state.rs`:

```rust
use nucleo::{Matcher, Utf32Str, pattern::{Pattern, CaseMatching, Normalization}};

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub label: String,           // Human-readable name: "Toggle Unified/Split View"
    pub keybinding: Option<String>, // Display hint: "Tab"
    pub action: crate::action::Action, // The action to dispatch
    pub category: PaletteCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteCategory {
    Navigation,
    DiffView,
    Annotations,
    Search,
    Git,
    Agent,
    Review,
    Settings,
}

impl PaletteCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::DiffView => "Diff View",
            Self::Annotations => "Annotations",
            Self::Search => "Search",
            Self::Git => "Git",
            Self::Agent => "Agent",
            Self::Review => "Review",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub cursor_pos: usize,
    pub selected_index: usize,
    pub filtered_entries: Vec<usize>, // indices into the master entry list
}
```

### Master Entry Registry

Create a function `build_palette_entries() -> Vec<PaletteEntry>` that registers ALL available actions with human-readable labels and keybinding hints. This should be called once at startup and cached. Example entries:

```rust
vec![
    PaletteEntry {
        label: "Jump to Next Hunk".into(),
        keybinding: Some("]".into()),
        action: Action::JumpNextHunk,
        category: PaletteCategory::DiffView,
    },
    PaletteEntry {
        label: "Toggle Unified/Split View".into(),
        keybinding: Some("Tab".into()),
        action: Action::ToggleViewMode,
        category: PaletteCategory::DiffView,
    },
    PaletteEntry {
        label: "Open Comment Editor".into(),
        keybinding: Some("c".into()),
        action: Action::OpenCommentEditor,
        category: PaletteCategory::Annotations,
    },
    PaletteEntry {
        label: "Export Feedback".into(),
        keybinding: Some("Ctrl+E".into()),
        action: Action::ExportFeedback,
        category: PaletteCategory::Review,
    },
    PaletteEntry {
        label: "Toggle Checklist Panel".into(),
        keybinding: Some("Ctrl+L".into()),
        action: Action::ToggleChecklist,
        category: PaletteCategory::Review,
    },
    PaletteEntry {
        label: "Open Settings".into(),
        keybinding: Some(",".into()),
        action: Action::OpenSettings,
        category: PaletteCategory::Settings,
    },
    // ... register all ~40 user-facing actions
]
```

**Important**: Only register user-facing actions. Internal actions like `Tick`, `Resize`, `CommentChar`, `SearchChar`, etc. should NOT appear in the palette.

### New Actions

Add to `src/action.rs`:

```rust
// Command palette
OpenCommandPalette,
CommandPaletteChar(char),
CommandPaletteBackspace,
CommandPaletteUp,
CommandPaletteDown,
CommandPaletteConfirm,
CancelCommandPalette,
```

### Key Mapping in `event.rs`

Add a new priority level (Priority 1.5 — after PTY focus but before other modals):

```rust
// Priority 1.5: Command palette (highest priority when open)
if ctx.command_palette_open {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => Some(Action::TextCursorHome),
            KeyCode::Char('e') => Some(Action::TextCursorEnd),
            KeyCode::Char('w') => Some(Action::TextDeleteWord),
            KeyCode::Char('n') => Some(Action::CommandPaletteDown),
            KeyCode::Char('p') => Some(Action::CommandPaletteUp),
            _ => None,
        };
    }
    return match key.code {
        KeyCode::Esc => Some(Action::CancelCommandPalette),
        KeyCode::Enter => Some(Action::CommandPaletteConfirm),
        KeyCode::Up => Some(Action::CommandPaletteUp),
        KeyCode::Down => Some(Action::CommandPaletteDown),
        KeyCode::Backspace => Some(Action::CommandPaletteBackspace),
        KeyCode::Left => Some(Action::TextCursorLeft),
        KeyCode::Right => Some(Action::TextCursorRight),
        KeyCode::Char(c) => Some(Action::CommandPaletteChar(c)),
        _ => None,
    };
}
```

Add the trigger in Priority 4 (global bindings):

```rust
KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    return Some(Action::OpenCommandPalette)
}
```

### Fuzzy Matching

Use the `nucleo` crate (already in `Cargo.toml`) for fuzzy matching. When the user types in the palette:

1. On each `CommandPaletteChar` / `CommandPaletteBackspace`:
   - Build a `Pattern` from the query string
   - Score each `PaletteEntry.label` against the pattern
   - Sort by score descending
   - Update `filtered_entries` with indices of matching entries
   - Reset `selected_index` to 0

2. When query is empty, show all entries grouped by category

### UI Component

Create `src/components/command_palette.rs`:

- Render as a centered floating panel (60% width, 50% height), similar to the agent selector modal
- Top: search input with blinking cursor and ">" prompt
- Below: scrollable list of matching entries
- Each entry shows: `[Category] Action Name          Keybinding`
- Selected entry highlighted with accent color
- Category labels in dim/muted color
- Keybinding right-aligned in a distinct color
- Match characters highlighted in the label (bold or accent color)
- Bottom: entry count "N commands" or "N/M matching"

Layout:
```
┌─ Command Palette ─────────────────────────────┐
│ > search query█                                │
│───────────────────────────────────────────────│
│ ▸ Toggle Unified/Split View           Tab      │
│   Jump to Next Hunk                   ]        │
│   Jump to Previous Hunk               [        │
│   Open Comment Editor                 c        │
│   Export Feedback                     Ctrl+E   │
│   Toggle Checklist Panel              Ctrl+L   │
│   Open Settings                       ,        │
│   ...                                          │
│                                                │
│ 42 commands                                    │
└────────────────────────────────────────────────┘
```

### Action Dispatch

When the user presses Enter (`CommandPaletteConfirm`):

1. Look up the selected entry from `filtered_entries[selected_index]`
2. Close the palette (`state.command_palette.open = false`)
3. Clone the entry's `Action` and dispatch it through the normal `handle_action` flow

This means selecting "Open Comment Editor" in the palette is identical to pressing `c` — it goes through the same code path.

### KeyContext Update

Add `command_palette_open: bool` to `KeyContext` in `event.rs`, populated from `state.command_palette.open`.

## Files to Modify

1. **`src/state/command_palette_state.rs`** (NEW): `CommandPaletteState`, `PaletteEntry`, `PaletteCategory`, `build_palette_entries()`
2. **`src/state/mod.rs`**: Add `pub mod command_palette_state;`
3. **`src/state/app_state.rs`**: Add `command_palette: CommandPaletteState` field to `AppState`
4. **`src/action.rs`**: Add command palette actions
5. **`src/event.rs`**: Add `command_palette_open` to `KeyContext`, add priority 1.5 handler, add `Ctrl+P` trigger
6. **`src/components/command_palette.rs`** (NEW): Floating panel UI component
7. **`src/components/mod.rs`**: Register new component
8. **`src/app.rs`**: Handle command palette actions, dispatch selected action, call component render

## Testing

1. Press `Ctrl+P` — palette opens as centered floating panel
2. Type "hunk" — list filters to show hunk-related actions
3. Arrow keys / `Ctrl+N` / `Ctrl+P` — navigate filtered list
4. Press `Enter` — selected action executes, palette closes
5. Press `Esc` — palette closes without executing
6. Type "export" — shows "Export Feedback" with `Ctrl+E` hint
7. Empty query — shows all commands grouped by category
8. Run `cargo check` — no compilation errors
9. Verify palette does not interfere with other modals (agent selector, settings, etc.)

## Edge Cases

- Opening palette while another modal is open: palette should take priority (close other modal first, or block opening)
- Actions that require specific context (e.g., `OpenCommentEditor` requires diff focus): dispatch normally and let existing guards handle invalid state
- Very long action labels: truncate with ellipsis to fit panel width
- Terminal resize while palette is open: re-center on next render

## Backward Compatibility

No existing keybindings are changed. `Ctrl+P` is currently unmapped. This is purely additive.
