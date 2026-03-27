# 032 — Automated Review Surface

## Summary

Build a review surface for AI-generated review comments produced by the agentic review system (#70). The agent organizes its findings as structured annotations with file paths, line ranges, severity levels, and categories. This feature presents those findings as a navigable, interactive review panel where users can accept, dismiss, or edit each AI finding.

## Motivation

Issue #71 requests this feature. After #70 landed agentic review mode on `main`, users can dump freeform feedback and have an AI agent organize it by areas of concern. However, the current implementation streams raw text output to a side panel — there's no structured navigation of individual findings. Users need to:

1. See AI-generated review comments organized by file and severity
2. Navigate between findings with keyboard shortcuts
3. Accept findings (converting them to persistent annotations)
4. Dismiss false positives
5. Edit AI suggestions before accepting them

This is the missing link between AI-generated analysis and mdiff's structured annotation system.

## User Stories

- **As a reviewer**, I want AI-generated review comments to appear as navigable items so I can triage them efficiently without reading a wall of text.
- **As a reviewer**, I want to accept an AI finding with a single key so it becomes a persistent annotation attached to the correct file and line range.
- **As a reviewer**, I want to dismiss false positives so they don't clutter my review.
- **As a reviewer**, I want to edit AI suggestions before accepting them so I can refine the feedback.

## Design

### Architecture

The feature builds on the existing `agentic_review` infrastructure:

1. **New state module**: `src/state/auto_review_state.rs`
   - `AutoReviewState` struct holding a `Vec<ReviewFinding>` parsed from agent output
   - Each `ReviewFinding` has: `file_path`, `old_range`, `new_range`, `comment`, `category`, `severity`, `status` (Pending/Accepted/Dismissed)
   - `selected_finding: usize` for navigation
   - `filter: Option<FindingFilter>` for filtering by status/severity/file

2. **Parsing layer**: Extend `src/agentic_review.rs`
   - After the `AgenticReviewComplete(Vec<Annotation>)` action fires, the annotations are already structured
   - Parse the completed annotations into `ReviewFinding` entries in `AutoReviewState`
   - Map annotation categories and severities to finding metadata

3. **New component**: `src/components/auto_review_panel.rs`
   - Renders in the right panel (replacing or alongside the existing agentic review stream panel)
   - Shows findings grouped by file, with severity color coding
   - Selected finding is highlighted; shows full comment in a detail area
   - Status indicators: pending (dot), accepted (checkmark), dismissed (x)

4. **Navigation & Actions**:
   - `Action::AutoReviewNext` / `Action::AutoReviewPrev` — navigate findings
   - `Action::AutoReviewAccept` — convert finding to persistent annotation
   - `Action::AutoReviewDismiss` — mark as dismissed (grayed out)
   - `Action::AutoReviewEdit` — open comment editor pre-filled with finding text
   - `Action::AutoReviewJumpToCode` — navigate diff view to the finding's location
   - `Action::AutoReviewFilterCycle` — cycle through filter modes

5. **Keybindings** (when auto-review panel is focused):
   - `j`/`k` or arrows — navigate findings
   - `Enter` — jump to finding location in diff view
   - `a` — accept finding (converts to annotation)
   - `x` — dismiss finding
   - `e` — edit before accepting
   - `f` — cycle filter (all/pending/accepted/dismissed)
   - `Esc` — close panel

### Integration Points

- **Action enum** (`src/action.rs`): Add `AutoReview*` variants
- **Event mapping** (`src/event.rs`): Map keys when `auto_review_panel_focused` context
- **App state** (`src/state/app_state.rs`): Add `auto_review: AutoReviewState` field
- **App handler** (`src/app.rs`): Handle new actions, wire up accept->annotation conversion
- **Which-key** (`src/components/which_key.rs`): Add context entries for auto-review panel
- **Existing agentic review**: The `AgenticReviewComplete` action handler populates `AutoReviewState`

### Data Flow

```
User triggers agentic review (R key)
  -> User types freeform feedback, confirms
  -> AgenticReviewRunner spawns orchestrator + child agents
  -> Streaming tokens shown in side panel
  -> AgenticReviewComplete(Vec<Annotation>) fires
  -> Annotations parsed into AutoReviewState.findings
  -> Auto-review panel renders findings
  -> User navigates, accepts/dismisses each finding
  -> Accepted findings -> AnnotationState (persistent)
```

## Acceptance Criteria

- [ ] AI-generated review findings display as a navigable list grouped by file
- [ ] Each finding shows severity badge, file path, line range, and truncated comment
- [ ] `j`/`k` navigation between findings
- [ ] `Enter` jumps diff view to the finding's file and line
- [ ] `a` converts a finding to a persistent annotation
- [ ] `x` dismisses a finding (visually grayed out, excluded from future exports)
- [ ] `e` opens comment editor pre-filled with finding text
- [ ] `f` cycles filter between all/pending/accepted/dismissed
- [ ] Finding count summary in panel header (e.g., "3/12 reviewed")
- [ ] Which-key overlay shows auto-review panel context
- [ ] `cargo check` passes
- [ ] `cargo test` passes (if tests exist)

## Files to Create/Modify

- **Create**: `src/state/auto_review_state.rs`
- **Create**: `src/components/auto_review_panel.rs`
- **Modify**: `src/action.rs` — add AutoReview action variants
- **Modify**: `src/event.rs` — add key mappings for auto-review context
- **Modify**: `src/state/app_state.rs` — add `auto_review` field
- **Modify**: `src/state/mod.rs` — re-export AutoReviewState
- **Modify**: `src/app.rs` — handle AutoReview actions, wire AgenticReviewComplete -> AutoReviewState
- **Modify**: `src/components/which_key.rs` — add auto-review context entries
