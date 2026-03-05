# Spec: Review Checklist Templates

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)

## Problem

When reviewing coding agent output, reviewers often need to check the same things every time: error handling, test coverage, no hardcoded secrets, proper logging, etc. Currently, mdiff collects freeform annotations and scores, but there's no structured way to ensure a reviewer has covered all the important dimensions before sending feedback. This leads to inconsistent review quality across sessions and missed issues.

Research on RLHF and structured feedback shows that checklists dramatically improve annotation consistency and reduce the "forgetting to check" failure mode. The bottleneck in AI-assisted development has shifted from code generation to code review — structured checklists help reviewers be thorough without being slow.

## Solution

Add a configurable review checklist system that:
1. Loads checklist templates from `~/.config/mdiff/config.toml` (per-project overrides via `.mdiff.toml` in repo root)
2. Displays a toggleable checklist panel alongside the diff view
3. Allows single-keypress toggling of checklist items during review
4. Includes checklist completion status in the prompt output sent to agents
5. Tracks checklist completion per-session for the feedback summary view

## Architecture

### New State: `ChecklistState` (src/state/checklist_state.rs)

```rust
#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub label: String,
    pub key: char,        // Single-key shortcut (1-9, a-z)
    pub checked: bool,
    pub note: Option<String>, // Optional reviewer note
}

#[derive(Debug, Clone, Default)]
pub struct ChecklistState {
    pub items: Vec<ChecklistItem>,
    pub selected: usize,
    pub panel_open: bool,
}
```

### New Actions (src/action.rs)

```rust
// Checklist
ToggleChecklist,        // Toggle checklist panel visibility
ChecklistUp,           // Navigate checklist items
ChecklistDown,
ChecklistToggleItem,   // Toggle current item checked/unchecked
ChecklistAddNote,      // Open note editor for current item
```

### Config Format (config.toml)

```toml
[checklist]
items = [
    { label = "Error handling verified", key = "e" },
    { label = "Tests cover edge cases", key = "t" },
    { label = "No hardcoded secrets", key = "s" },
    { label = "Logging is appropriate", key = "l" },
    { label = "Types are correct", key = "y" },
    { label = "No unnecessary complexity", key = "c" },
]
```

### Keybindings

- `C` (in DiffExplorer, not visual mode): Toggle checklist panel
- When checklist panel is focused:
  - `j`/`k` or Up/Down: Navigate items
  - `Space` or `Enter`: Toggle item checked/unchecked
  - `n`: Add/edit note on current item
  - `Esc`: Close panel / return focus to diff

### Component: `ChecklistPanel` (src/components/checklist_panel.rs)

Renders as a right-side panel (similar to prompt preview) showing:
- Each checklist item with `[x]` or `[ ]` prefix
- The shortcut key in a dimmed style
- Optional notes in italic
- A progress bar at the top: "3/6 items checked"
- Color coding: unchecked items in yellow, checked in green

### Integration Points

1. **Prompt output** (`src/prompt.rs`): Include checklist status in the generated prompt:
   ```
   ## Review Checklist
   - [x] Error handling verified
   - [ ] Tests cover edge cases (Note: only happy path tested)
   - [x] No hardcoded secrets
   ```

2. **Feedback summary** (PR #15): Show checklist completion percentage alongside annotation stats.

3. **Session persistence** (`src/session.rs`): Save checklist state in the session file so reviews can be resumed.

4. **HUD/Which-key**: Register `C` binding in the which-key overlay for discoverability.

### Config Loading

1. Check for `.mdiff.toml` in the current repo root (project-specific)
2. Fall back to `~/.config/mdiff/config.toml` (global)
3. If no checklist config exists, don't show the checklist feature (graceful absence)

Config loading should happen in `src/config.rs` (create if needed) at app startup, populating `ChecklistState` in `AppState`.

## Edge Cases

- If no checklist is configured, the `C` key should show a brief message: "No checklist configured. Add [checklist] to config.toml"
- Checklist items should be limited to 20 max (UI constraint)
- Notes should be limited to 200 characters (single line in the panel)
- When a session is loaded, checklist state is restored; when a new session starts, checklist resets

## Testing

- Unit test: ChecklistState toggle logic
- Unit test: Config parsing with valid/invalid/missing checklist sections
- Unit test: Prompt output includes checklist when items exist
- Manual test: Panel renders correctly at various terminal sizes
