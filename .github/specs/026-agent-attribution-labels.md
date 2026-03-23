# Spec: Agent Attribution Labels (Hunk-Level Agent Session Markers)

**Priority**: P1 (High Impact — Core Mission Differentiator)
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)
**Inspiration**: Muse VCS conflict-aware diff labels (Manyana pattern), oh-my-pi agentic git tools

## Problem

When reviewing coding agent output, users often work with changesets produced across multiple agent sessions or iterative agent runs. Currently, mdiff treats all hunks uniformly — there is no visual signal indicating *which agent session* produced a particular change. For iterative workflows where a user runs an agent, reviews, provides feedback, and runs again, it's impossible to distinguish "original agent changes" from "follow-up corrections" without mentally tracking commit boundaries.

This is analogous to the merge conflict UX problem that Muse/Manyana solves: instead of opaque "ours vs theirs" blobs, label each hunk with the action and which side performed it. For mdiff, the "sides" are different agent sessions.

## Design

### Data Model

Add an `AgentAttribution` struct to track which agent session produced each hunk:

```rust
// src/state/attribution_state.rs (new file)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentSession {
    /// Display label (e.g., "Session #1", "Claude 2026-03-20 14:30", commit SHA prefix)
    pub label: String,
    /// Unique identifier (commit SHA or session ID)
    pub id: String,
    /// Color index for visual differentiation (0-7, cycles through palette)
    pub color_index: u8,
}

#[derive(Debug, Default)]
pub struct AttributionState {
    /// Map from (file_path, hunk_index) -> AgentSession
    pub hunk_attributions: HashMap<(String, usize), AgentSession>,
    /// All known agent sessions in order
    pub sessions: Vec<AgentSession>,
    /// Whether attribution mode is active
    pub active: bool,
}
```

### Attribution Detection

Attribution is determined by analyzing the commit history of the diff target:

1. **Commit-based attribution**: When diffing against a base (e.g., `main`), each commit between base and HEAD is treated as an agent session. Hunks are mapped to the commit that introduced them using `git log --follow -p`.
2. **Worktree-based attribution**: When comparing worktrees (existing mdiff feature), each worktree's changes are attributed to that worktree's session.
3. **Manual attribution**: Users can tag hunks with `Ctrl+T` to manually assign attribution labels.

### UI Elements

1. **Gutter attribution markers**: In the line number gutter, show a colored dot or session label (e.g., `[S1]`, `[S2]`) for each line, colored by session.
2. **Hunk header labels**: Extend the hunk header line (e.g., `@@ -10,5 +10,7 @@`) with an attribution tag: `@@ -10,5 +10,7 @@ [Session #1: "Add error handling"]`.
3. **Session legend**: A compact legend at the top of the diff view showing session colors and labels.
4. **Filter by session**: Press `A` in diff view to cycle through sessions, showing only hunks from a specific session (or all).

### Keybindings

| Key | Context | Action |
|-----|---------|--------|
| `A` | Diff View | Cycle attribution filter (All → Session 1 → Session 2 → ... → All) |
| `Shift+A` | Diff View | Toggle attribution display on/off |
| `Ctrl+T` | Visual Mode | Tag selected lines with a session label |

### Actions

```rust
// Add to src/action.rs
ToggleAttribution,          // Shift+A: toggle attribution display
CycleAttributionFilter,     // A: cycle through session filters
TagAttribution(String),     // Ctrl+T: manually tag selected lines
```

### Integration Points

- **Navigator**: Show per-file session breakdown (e.g., "3 files from Session #1, 2 from Session #2")
- **Feedback Summary**: Group annotations by session in the export
- **Diff Statistics Dashboard** (spec 019): Show per-session stats
- **Export**: Include attribution metadata in feedback JSON export

### Color Palette

Use 8 distinct colors that are visible on both dark and light terminal backgrounds, cycling for sessions beyond 8:

```rust
const SESSION_COLORS: [Color; 8] = [
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
    Color::Green,
    Color::Blue,
    Color::Red,
    Color::LightCyan,
    Color::LightYellow,
];
```

## Implementation Steps

1. Create `src/state/attribution_state.rs` with `AgentSession`, `AttributionState`
2. Add `attribution: AttributionState` to `AppState`
3. Add `ToggleAttribution`, `CycleAttributionFilter`, `TagAttribution` to `Action` enum
4. Implement commit-based attribution detection in a new `src/git/attribution.rs` module
5. Update `src/components/diff_view.rs` to render gutter markers and hunk header labels
6. Update `src/event.rs` with keybinding mappings (`A`, `Shift+A`, `Ctrl+T`)
7. Update `src/components/which_key.rs` with attribution entries
8. Update `src/export.rs` to include attribution in feedback export

## Edge Cases

- **Single-commit diffs**: Attribution is trivial (one session). Display is still useful as confirmation.
- **Merge commits**: Attribution follows the merge parent chain. Complex merges may have ambiguous attribution.
- **Uncommitted changes**: Treated as a special "Working Directory" session.
- **Performance**: Attribution detection runs once when loading the diff, not on every render. Cache results in `AttributionState`.

## User Benefit

Transforms mdiff from a generic diff viewer into a purpose-built agent review tool. When a reviewer sees 50 files changed across 3 agent iterations, they can instantly filter to "what did the agent change in its last run?" — reducing review scope by 60-70% for iterative workflows. Attribution data in exports also enables tracking agent improvement over time.
