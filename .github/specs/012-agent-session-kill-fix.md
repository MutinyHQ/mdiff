# Spec: Fix cmd+K Killing Wrong Agent Session (Issue #24)

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Small (1-2 files changed)
**Addresses**: GitHub Issue #24

## Problem

When there are multiple agent sessions in the Agent Outputs tab, pressing `Ctrl+K` (kill agent) doesn't kill the currently focused/selected session. Instead, it kills a different session, causing confusion. The user expects the kill command to target the session they're currently viewing, but the index mapping between the selected session and the actual kill target is misaligned.

## Root Cause Analysis

In `src/app.rs`, the `Action::KillAgentProcess` handler likely uses an incorrect index to determine which agent to kill. The Agent Outputs state tracks `selected` as an index into the visible/displayed list, but the kill logic may be using this index against the raw `runs` vector without accounting for:

1. **Display ordering**: The list might be displayed in reverse chronological order, but the `selected` index maps to the display order while the kill logic uses the underlying `runs` vector order.
2. **Filtering**: If completed/failed runs are filtered from display but not from the backing vector, the selected index and the backing vector index diverge.
3. **Off-by-one**: The `selected` index might be 0-based from the top of the displayed list but the kill logic indexes from a different reference point.

## Solution

### Audit and fix the index mapping in KillAgentProcess handler

#### File: `src/state/agent_state.rs`

Examine the `AgentOutputsState` structure. The `selected` field should correspond to the currently highlighted item in the Agent Outputs list. Add a helper method to resolve the selected index to the correct run:

```rust
impl AgentOutputsState {
    /// Get the currently selected agent run, accounting for display ordering.
    pub fn selected_run(&self) -> Option<&AgentRun> {
        self.runs.get(self.selected)
    }

    /// Get a mutable reference to the currently selected agent run.
    pub fn selected_run_mut(&mut self) -> Option<&mut AgentRun> {
        self.runs.get_mut(self.selected)
    }
}
```

#### File: `src/app.rs`

In the `Action::KillAgentProcess` handler, ensure the kill targets the correct run:

```rust
Action::KillAgentProcess => {
    // Use the selected index to find the correct PTY to kill
    if let Some(run) = self.state.agent_outputs.selected_run_mut() {
        if run.status == AgentRunStatus::Running {
            if let Some(pty_id) = &run.pty_id {
                // Kill the PTY process associated with this specific run
                // ... kill logic using the correct pty_id
            }
            run.status = AgentRunStatus::Killed;
            self.state.status_message = Some((
                format!("Killed agent: {}", run.label),
                false,
            ));
        } else {
            self.state.status_message = Some((
                "Selected agent is not running".to_string(),
                true,
            ));
        }
    }
    self.hud_collapse_countdown = 100;
}
```

The key insight is that the kill logic must use the **same index resolution** as the display/rendering logic. If the Agent Outputs component renders runs in a specific order (e.g., reversed, or filtered), the kill handler must apply the same transformation to `selected` before indexing into the backing vector.

#### File: `src/components/agent_outputs.rs`

Verify that the rendering order matches the state order. If the component renders `runs` in reverse order or applies any filtering, document that mapping clearly and ensure the `selected` index follows the same convention.

### Investigation Steps for Implementation

1. Read `src/state/agent_state.rs` — understand the `runs` vector ordering and `selected` semantics
2. Read `src/components/agent_outputs.rs` — understand how runs are displayed (order, filtering)
3. Read the `KillAgentProcess` handler in `src/app.rs` — identify the mismatch
4. Read the `AgentOutputsUp`/`AgentOutputsDown` handlers — verify `selected` increments correctly
5. Fix the index mapping so kill targets the displayed selection

## Files Changed

- `src/app.rs` — Fix `KillAgentProcess` handler index mapping
- `src/state/agent_state.rs` — Add `selected_run()` / `selected_run_mut()` helpers if needed
- `src/components/agent_outputs.rs` — Verify/document display ordering

## Testing

1. `cargo check` passes
2. Open mdiff with multiple agent sessions running
3. Select the second session using `j`/`k` navigation
4. Press `Ctrl+K` — verify the **selected** session is killed, not another one
5. Verify the status message shows the correct agent name
6. Verify navigation still works after killing a session (selected index doesn't go out of bounds)
