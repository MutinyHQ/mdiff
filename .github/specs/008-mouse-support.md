# Spec: Mouse Support for Navigation

**Priority**: P2
**Status**: Ready for implementation
**Estimated effort**: Small-Medium (3-5 files changed)

## Problem

mdiff captures mouse events in `event.rs` (`Event::Mouse(MouseEvent)`) but does not process them. While the keyboard-driven workflow is mdiff's core UX, many developers use the mouse as a secondary input for quick navigation — clicking a file in a list, scrolling with the wheel, or clicking a line to select it. Competitors like lazygit and VS Code support mouse interaction alongside keyboard shortcuts. Not supporting mouse feels like a gap, especially for users new to the tool.

## Solution

Add mouse support for the three most impactful interactions:
1. **Scroll wheel** in diff view and navigator (most requested)
2. **Click to select** files in the navigator
3. **Click to position cursor** in the diff view

This is intentionally scoped to navigation only — annotation and editing flows remain keyboard-driven to preserve the structured feedback workflow.

## Architecture

### Event Handling (src/event.rs)

Currently, `Event::Mouse(MouseEvent)` is captured but not mapped to actions. Add a new function:

```rust
use crossterm::event::{MouseEventKind, MouseButton};

/// Map a mouse event to an action based on current app context.
pub fn map_mouse_to_action(mouse: MouseEvent, ctx: &MouseContext) -> Option<Action> {
    match mouse.kind {
        // Scroll wheel
        MouseEventKind::ScrollUp => {
            match ctx.panel_at(mouse.column, mouse.row) {
                Some(Panel::Navigator) => Some(Action::NavigatorUp),
                Some(Panel::DiffView) => Some(Action::ScrollUp),
                _ => None,
            }
        }
        MouseEventKind::ScrollDown => {
            match ctx.panel_at(mouse.column, mouse.row) {
                Some(Panel::Navigator) => Some(Action::NavigatorDown),
                Some(Panel::DiffView) => Some(Action::ScrollDown),
                _ => None,
            }
        }
        // Left click
        MouseEventKind::Down(MouseButton::Left) => {
            match ctx.panel_at(mouse.column, mouse.row) {
                Some(Panel::Navigator) => {
                    let file_index = ctx.navigator_row_to_index(mouse.row);
                    file_index.map(Action::SelectFile)
                }
                Some(Panel::DiffView) => {
                    // Click to focus diff view + position cursor
                    Some(Action::FocusDiffView)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
```

### New Types

```rust
/// Context for mouse event mapping.
pub struct MouseContext {
    pub navigator_rect: Rect,
    pub diff_view_rect: Rect,
    pub navigator_scroll_offset: usize,
    pub navigator_item_count: usize,
}

enum Panel {
    Navigator,
    DiffView,
}

impl MouseContext {
    /// Determine which panel a screen coordinate falls in.
    fn panel_at(&self, col: u16, row: u16) -> Option<Panel> {
        if self.navigator_rect.contains((col, row).into()) {
            Some(Panel::Navigator)
        } else if self.diff_view_rect.contains((col, row).into()) {
            Some(Panel::DiffView)
        } else {
            None
        }
    }

    /// Convert a mouse row in the navigator area to a file index.
    fn navigator_row_to_index(&self, row: u16) -> Option<usize> {
        let relative_row = row.saturating_sub(self.navigator_rect.y + 1); // +1 for border
        let index = self.navigator_scroll_offset + relative_row as usize;
        if index < self.navigator_item_count {
            Some(index)
        } else {
            None
        }
    }
}
```

### App Integration (src/app.rs)

In the main event loop where `Event::Mouse` is currently a no-op:

```rust
Event::Mouse(mouse) => {
    let ctx = MouseContext {
        navigator_rect: self.last_navigator_rect,
        diff_view_rect: self.last_diff_view_rect,
        navigator_scroll_offset: self.state.navigator.scroll_offset,
        navigator_item_count: self.state.navigator.entries.len(),
    };
    if let Some(action) = map_mouse_to_action(mouse, &ctx) {
        self.handle_action(action);
    }
}
```

This requires storing the layout rects from the last render pass. Add fields to `App`:

```rust
pub struct App {
    // ... existing fields ...
    last_navigator_rect: Rect,
    last_diff_view_rect: Rect,
}
```

Update the render function to save these rects after layout computation.

### Configuration (config.toml)

```toml
[mouse]
enabled = true  # Default: true. Set to false to disable mouse handling entirely.
```

### Scroll Acceleration

For scroll wheel events, map multiple scroll ticks to faster scrolling:
- Single scroll tick: Move 1 line (NavigatorUp/Down or ScrollUp/Down)
- This matches the existing keyboard behavior and feels natural

Future enhancement: Hold Shift + scroll for page-level scrolling.

## Keybindings / Actions

No new actions needed — mouse events map to existing actions:
- `ScrollUp/Down` for wheel in diff
- `NavigatorUp/Down` for wheel in navigator
- `SelectFile(index)` for click in navigator
- `FocusDiffView` / `FocusNavigator` for click to focus

## Edge Cases

- Mouse events during modal dialogs (commit, comment editor, etc.): Ignore mouse events when any modal is open
- Mouse events during search mode: Ignore (keyboard takes priority)
- Terminal resizing while mouse is in use: Rects update on next render, mouse coordinates may be stale for one frame — acceptable
- Terminals that don't support mouse: crossterm handles this gracefully; mouse events simply won't arrive
- Click on panel borders: Treat as no-op (panel_at returns None for border pixels)

## Testing

- Manual test: Scroll wheel in navigator scrolls file list
- Manual test: Scroll wheel in diff view scrolls diff content
- Manual test: Click file in navigator selects it and shows diff
- Manual test: Mouse disabled in config.toml suppresses all mouse handling
- Manual test: Mouse during modal dialogs is properly ignored
