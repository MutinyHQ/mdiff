# 034 — Inline Diff Minimap

## Summary

Add a vertical minimap on the right edge of the diff view showing change density across the entire file. Color-coded bars indicate additions (green), deletions (red), and mixed changes (yellow). The current viewport position is highlighted as a bright marker. Toggled with `M` key.

## Motivation

When reviewing large files modified by coding agents, reviewers lose orientation. They scroll through hundreds of lines without knowing where changes are concentrated or how much of the file remains. VS Code's minimap is one of its most-used navigation features for exactly this reason. No TUI diff viewer currently offers this.

The minimap provides instant visual signal about change distribution — a dense red cluster might indicate a large deletion worth investigating, while scattered green dots suggest additions throughout the file. Combined with click-to-jump (via mouse), it turns the minimap into a navigation aid.

This was previously listed as a P2 item ("Inline Diff Minimap") in the roadmap. Promoting to implementation based on growing file sizes in agent output and validation from the competitive landscape (patchcast's visual approach, VS Code minimap).

## User Stories

- **As a reviewer**, I want to see at a glance where changes are concentrated in a large file so I can navigate directly to the most important sections.
- **As a reviewer**, I want to know my current viewport position relative to the whole file so I understand how much review remains.
- **As a reviewer**, I want to see change density patterns (clustered vs scattered) to quickly assess the agent's modification strategy.

## Design

### Architecture

1. **New state module**: `src/state/minimap_state.rs`
   - `MinimapState` struct:
     - `visible: bool` — toggle state
     - `width: u16` — minimap column width (default 3)
   - Simple state, most logic is in the rendering component

2. **New component**: `src/components/minimap.rs`
   - Renders a narrow vertical column (3 chars wide) on the right edge of the diff view
   - Each row in the minimap maps to a proportional section of the full file
   - Color mapping:
     - Addition lines: green block (or full block character)
     - Deletion lines: red block
     - Mixed (both add and delete in the mapped section): yellow block
     - Context (no changes): dim/empty
     - Current viewport: bright accent color background or inverse
   - Viewport indicator: A highlighted region showing which portion of the file is currently visible
   - The minimap is rendered inside the diff view's layout, stealing 3 columns from the right

3. **Density calculation**:
   - Given total file lines `N` and minimap height `H`, each minimap row represents `ceil(N/H)` file lines
   - For each minimap row, scan the corresponding line range in the display map
   - Count additions, deletions, and context lines in that range
   - Choose the dominant color (addition-heavy: green, deletion-heavy: red, mixed: yellow)
   - Intensity can be encoded via Unicode block characters: light shade (sparse), medium shade (medium), full block (dense)

4. **Layout integration**: Modify `src/components/diff_view.rs`
   - When minimap is visible, split the diff area: `[diff_content | minimap(3)]`
   - The diff content area shrinks by 3 columns
   - Minimap renders in the rightmost 3 columns

5. **Actions** (in `src/action.rs`):
   - `Action::ToggleMinimap` — toggle minimap visibility

6. **Keybindings**:
   - `M` — toggle minimap (when not in visual mode and focus is on DiffView)

7. **Mouse support** (if mouse is enabled):
   - Clicking on a minimap row scrolls the diff view to the corresponding position
   - This integrates with existing mouse handling in `src/app.rs`

### Integration Points

- **Diff view layout** (`src/components/diff_view.rs`): Minimap is rendered as a right-side column within the diff view, not as a separate panel. This keeps the layout clean and doesn't affect focus management.
- **App state** (`src/state/app_state.rs`): Add `minimap: MinimapState` field.
- **Display map**: The minimap reads from the existing display map (computed for rendering) to classify each line.
- **Mouse handler** (`src/app.rs`): If mouse is enabled, clicks in the minimap column should scroll to the corresponding file position.
- **Config** (`src/config.rs`): Add `minimap` section for default visibility and width.

## Acceptance Criteria

- [ ] `M` key toggles the minimap column on the right edge of the diff view
- [ ] Minimap correctly maps each row to a proportional section of the file
- [ ] Addition-heavy sections show green, deletion-heavy show red, mixed show yellow
- [ ] Current viewport position is clearly indicated with a highlight
- [ ] Minimap resizes correctly with terminal window changes
- [ ] Diff view content area shrinks by minimap width when visible
- [ ] Works in both unified and split view modes
- [ ] Empty files / files with no changes show a dim/empty minimap
- [ ] `cargo check` passes
- [ ] `cargo test` passes (if tests exist)

## Files to Create/Modify

- **Create**: `src/state/minimap_state.rs`
- **Create**: `src/components/minimap.rs`
- **Modify**: `src/action.rs` — add `ToggleMinimap` action
- **Modify**: `src/event.rs` — add `M` key mapping
- **Modify**: `src/state/app_state.rs` — add `minimap: MinimapState` field
- **Modify**: `src/state/mod.rs` — re-export MinimapState
- **Modify**: `src/components/diff_view.rs` — integrate minimap into layout
- **Modify**: `src/components/which_key.rs` — add minimap entry in diff view context
- **Modify**: `src/config.rs` — add minimap config option
