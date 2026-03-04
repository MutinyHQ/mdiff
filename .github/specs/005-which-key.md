# Spec: Contextual Help Overlay (Which-Key)

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Medium (4-6 files changed)

## Problem

mdiff has 50+ keybindings spread across 8+ distinct contexts: Navigator, DiffView, Visual Mode, Comment Editor, Commit Dialog, Agent Selector, Worktree Browser, Agent Outputs, Settings Modal, Search Mode, etc. The existing HUD (`?` toggle) shows a static help panel, but it doesn't adapt to the current context. Users frequently forget which keys are available in which mode.

Every major keyboard-driven TUI solves this:
- **Neovim (which-key.nvim)**: Shows available keys after a delay when a prefix key is pressed
- **Helix**: Context-sensitive help in the status bar
- **Lazygit**: Panel-specific key legend at the bottom
- **Emacs (which-key)**: Popup showing continuations after a prefix

mdiff needs a lightweight, context-aware overlay that shows exactly which keys are available *right now* based on the current focus, active view, and modal state.

## Design

### Behavior

1. The overlay appears automatically when the user presses `?` (replacing the current static HUD toggle)
2. It shows only the keybindings relevant to the current context
3. It renders as a floating panel in the bottom-right area of the screen
4. Dismissed by pressing any key (the key's action still fires)
5. Can be disabled via config: `[ui] which_key = false`

### Layout

Compact grid layout, 2-3 columns depending on terminal width:

```
┌─ DiffView ──────────────────────────────────┐
│ j/k  Scroll up/down    v  Visual select     │
│ g/G  Top/bottom        i  Add annotation    │
│ h    Focus navigator   a  Annotation menu   │
│ /    Search in diff     p  Prompt preview    │
│ n/N  Next/prev match   y  Copy prompt       │
│ ]    Next annotation   [  Prev annotation   │
│ s    Stage file        u  Unstage file      │
│ r    Restore file      c  Commit            │
│ w    Toggle whitespace Tab Split/unified    │
│ R    Refresh           F  Feedback summary  │
│ 1-5  Quick score       0  Remove score      │
│ ?    This help         q  Quit              │
└─────────────────────────────────────────────┘
```

Different content for each context:

**Navigator:**
```
┌─ Navigator ─────────────────────────────────┐
│ j/k  Navigate files    /  Search files      │
│ g/G  Top/bottom        m  Mark reviewed     │
│ l/→  Focus diff view   n  Next unreviewed   │
│ s    Stage file        u  Unstage file      │
│ ...                                          │
└─────────────────────────────────────────────┘
```

**Visual Mode:**
```
┌─ Visual Mode ───────────────────────────────┐
│ j/k  Extend selection  i  Add annotation    │
│ d    Delete annotation y  Copy prompt       │
│ 1-5  Quick score       v/Esc  Exit visual   │
└─────────────────────────────────────────────┘
```

### Context Detection

The overlay reads the same `KeyContext` struct used by `map_key_to_action` to determine which bindings to show. This ensures the help is always perfectly in sync with the actual key handling.

## Implementation

### 1. `src/state/app_state.rs` — Add which-key state

```rust
// Which-key overlay
pub which_key_visible: bool,
```

Initialize to `false` in `AppState::new()`.

### 2. `src/action.rs` — Add actions

```rust
// Which-key help overlay
ToggleWhichKey,
DismissWhichKey,
```

### 3. `src/event.rs` — Update keybindings

**Replace existing `?` binding:** Change the `ToggleHud` binding to `ToggleWhichKey`:

```rust
// In Priority 6 (Diff explorer global bindings):
KeyCode::Char('?') => return Some(Action::ToggleWhichKey),
```

**Important:** When the which-key overlay is visible, ANY keypress should dismiss it AND process the key normally. This is handled in app.rs by checking the flag, not by intercepting here.

### 4. `src/components/which_key.rs` — New component

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::app_state::{ActiveView, FocusPanel};
use crate::state::AppState;

/// A single keybinding entry for display.
struct KeyEntry {
    key: &'static str,
    description: &'static str,
}

pub fn render_which_key(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.which_key_visible {
        return;
    }

    let entries = get_context_entries(state);
    if entries.is_empty() {
        return;
    }

    // Calculate overlay size
    let max_key_width = entries.iter().map(|e| e.key.len()).max().unwrap_or(3);
    let max_desc_width = entries.iter().map(|e| e.description.len()).max().unwrap_or(10);
    let entry_width = max_key_width + max_desc_width + 3; // key + "  " + desc

    // Two-column layout if enough entries
    let (cols, rows) = if entries.len() > 10 {
        (2, (entries.len() + 1) / 2)
    } else {
        (1, entries.len())
    };

    let panel_width = (entry_width * cols + 4).min(area.width as usize) as u16;
    let panel_height = (rows + 2).min(area.height as usize) as u16; // +2 for borders

    // Position: bottom-right of the screen
    let x = area.x + area.width.saturating_sub(panel_width + 1);
    let y = area.y + area.height.saturating_sub(panel_height + 1);
    let overlay_area = Rect::new(x, y, panel_width, panel_height);

    // Clear background
    frame.render_widget(Clear, overlay_area);

    // Determine title based on context
    let title = get_context_title(state);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.accent));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    // Build lines
    let mut lines: Vec<Line> = Vec::new();

    if cols == 2 {
        let half = (entries.len() + 1) / 2;
        for i in 0..half {
            let mut spans = Vec::new();

            // Left column
            spans.push(Span::styled(
                format!("{:>width$}", entries[i].key, width = max_key_width),
                Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!("  {:<width$}", entries[i].description, width = max_desc_width),
                Style::default().fg(state.theme.text),
            ));

            // Right column
            if i + half < entries.len() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("{:>width$}", entries[i + half].key, width = max_key_width),
                    Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    format!("  {}", entries[i + half].description),
                    Style::default().fg(state.theme.text),
                ));
            }

            lines.push(Line::from(spans));
        }
    } else {
        for entry in &entries {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>width$}", entry.key, width = max_key_width),
                    Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", entry.description),
                    Style::default().fg(state.theme.text),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn get_context_title(state: &AppState) -> &'static str {
    if state.selection.active {
        return "Visual Mode";
    }
    match state.active_view {
        ActiveView::WorktreeBrowser => "Worktree Browser",
        ActiveView::AgentOutputs => "Agent Outputs",
        ActiveView::FeedbackSummary => "Feedback Summary",
        ActiveView::DiffExplorer => match state.focus {
            FocusPanel::Navigator => "Navigator",
            FocusPanel::DiffView => "Diff View",
        },
    }
}

fn get_context_entries(state: &AppState) -> Vec<KeyEntry> {
    // Return entries based on current context, matching the actual keybindings in event.rs

    if state.selection.active {
        return vec![
            KeyEntry { key: "j/k", description: "Extend selection" },
            KeyEntry { key: "i", description: "Add annotation" },
            KeyEntry { key: "d", description: "Delete annotation" },
            KeyEntry { key: "y", description: "Copy prompt" },
            KeyEntry { key: "1-5", description: "Quick score" },
            KeyEntry { key: "v/Esc", description: "Exit visual" },
        ];
    }

    match state.active_view {
        ActiveView::WorktreeBrowser => vec![
            KeyEntry { key: "j/k", description: "Navigate" },
            KeyEntry { key: "Enter", description: "Select worktree" },
            KeyEntry { key: "r", description: "Refresh" },
            KeyEntry { key: "f", description: "Freeze" },
            KeyEntry { key: "Esc", description: "Back" },
        ],
        ActiveView::AgentOutputs => vec![
            KeyEntry { key: "j/k", description: "Navigate" },
            KeyEntry { key: "y", description: "Copy prompt" },
            KeyEntry { key: "w", description: "Switch worktree" },
            KeyEntry { key: "Enter", description: "PTY focus" },
            KeyEntry { key: "Ctrl+K", description: "Kill agent" },
        ],
        ActiveView::FeedbackSummary => vec![
            KeyEntry { key: "j/k", description: "Scroll" },
            KeyEntry { key: "y", description: "Copy JSON" },
            KeyEntry { key: "p", description: "Copy prompt" },
            KeyEntry { key: "Esc/F", description: "Close" },
        ],
        ActiveView::DiffExplorer => match state.focus {
            FocusPanel::Navigator => vec![
                KeyEntry { key: "j/k", description: "Navigate files" },
                KeyEntry { key: "g/G", description: "Top/bottom" },
                KeyEntry { key: "l/Enter", description: "Focus diff" },
                KeyEntry { key: "/", description: "Search files" },
                KeyEntry { key: "m", description: "Mark reviewed" },
                KeyEntry { key: "n", description: "Next unreviewed" },
                KeyEntry { key: "s", description: "Stage file" },
                KeyEntry { key: "u", description: "Unstage file" },
                KeyEntry { key: "r", description: "Restore file" },
                KeyEntry { key: "c", description: "Commit" },
                KeyEntry { key: "t", description: "Change target" },
                KeyEntry { key: "o", description: "Agent outputs" },
                KeyEntry { key: "Ctrl+W", description: "Worktrees" },
                KeyEntry { key: "Ctrl+A", description: "Agent selector" },
                KeyEntry { key: "Tab", description: "Split/unified" },
                KeyEntry { key: "R", description: "Refresh" },
                KeyEntry { key: "F", description: "Feedback summary" },
                KeyEntry { key: ":", description: "Settings" },
                KeyEntry { key: "?", description: "This help" },
                KeyEntry { key: "q", description: "Quit" },
            ],
            FocusPanel::DiffView => vec![
                KeyEntry { key: "j/k", description: "Scroll" },
                KeyEntry { key: "g/G", description: "Top/bottom" },
                KeyEntry { key: "h", description: "Focus navigator" },
                KeyEntry { key: "PgUp/Dn", description: "Page scroll" },
                KeyEntry { key: "Space", description: "Expand context" },
                KeyEntry { key: "/", description: "Search in diff" },
                KeyEntry { key: "n/N", description: "Next/prev match" },
                KeyEntry { key: "v", description: "Visual select" },
                KeyEntry { key: "i", description: "Add annotation" },
                KeyEntry { key: "a", description: "Annotation menu" },
                KeyEntry { key: "]", description: "Next annotation" },
                KeyEntry { key: "[", description: "Prev annotation" },
                KeyEntry { key: "p", description: "Prompt preview" },
                KeyEntry { key: "y", description: "Copy prompt" },
                KeyEntry { key: "1-5", description: "Quick score" },
                KeyEntry { key: "0", description: "Remove score" },
                KeyEntry { key: "s", description: "Stage file" },
                KeyEntry { key: "u", description: "Unstage file" },
                KeyEntry { key: "w", description: "Toggle whitespace" },
                KeyEntry { key: "Tab", description: "Split/unified" },
                KeyEntry { key: "F", description: "Feedback summary" },
                KeyEntry { key: "?", description: "This help" },
                KeyEntry { key: "q", description: "Quit" },
            ],
        },
    }
}
```

### 5. `src/components/mod.rs` — Register component

Add: `pub mod which_key;`

### 6. `src/app.rs` — Handle actions and auto-dismiss

```rust
Action::ToggleWhichKey => {
    self.state.which_key_visible = !self.state.which_key_visible;
}
```

**Auto-dismiss logic:** In the main event processing loop, before dispatching the action, check if which-key is visible and dismiss it. The key insight is that `?` toggles, and any other key dismisses:

```rust
// In the event loop, after mapping key to action:
if let Some(action) = action {
    // Auto-dismiss which-key on any keypress (except the ? toggle itself)
    if self.state.which_key_visible && !matches!(action, Action::ToggleWhichKey) {
        self.state.which_key_visible = false;
        // Still process the action normally — fall through
    }
    
    self.handle_action(action);
}
```

### 7. Main render function — Render overlay on top

In the main render function, add the which-key overlay LAST so it renders on top of everything:

```rust
// Render which-key overlay (must be last)
which_key::render_which_key(frame, frame.size(), &self.state);
```

### 8. Config support (optional enhancement)

In `~/.config/mdiff/config.toml`:

```toml
[ui]
which_key = true  # Set to false to disable the which-key overlay
```

Read this in the config loading logic and skip rendering if disabled.

## Migration Notes

- The `?` key currently maps to `ToggleHud`. This spec replaces that with `ToggleWhichKey`. The old HUD behavior is superseded by the context-aware which-key overlay.
- If the old HUD should be preserved as well, it could be moved to a different key (e.g., `Ctrl+?`), but this is not recommended — the which-key overlay is strictly better.

## Testing

- Press `?` in Navigator — verify Navigator-specific bindings appear
- Press `l` to focus DiffView, then `?` — verify DiffView-specific bindings appear
- Press `v` to enter visual mode, then `?` — verify Visual Mode bindings appear
- Press any key while overlay is showing — verify overlay dismisses AND the key action fires
- Press `?` twice — verify toggle on/off works
- Switch to Worktree Browser (`Ctrl+W`) — press `?` — verify Worktree bindings appear
- Verify overlay renders in bottom-right and doesn't overflow the terminal
- Test on small terminal (80x24) — verify layout adapts
