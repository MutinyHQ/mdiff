# Spec: Approve/Reject Agent Workflow

**Priority**: P1 (New)
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)

## Problem

After reviewing an agent's changeset — leaving annotations, scores, and checking off review items — there is no explicit decision point. The reviewer finishes, copies the prompt to clipboard, and... that's it. There's no clear signal of "I approve this changeset" or "I reject this and here's why." The feedback loop is incomplete.

Research on IPE (a Claude Code review interface) and GitHub's own PR review model shows that an explicit approve/reject decision dramatically improves the feedback loop:
- It forces the reviewer to make a deliberate judgment call
- It structures the exported feedback with a clear verdict
- It enables tracking review outcomes over time (approval rate per agent, per project)
- It creates a natural end-of-review ritual that prevents half-finished reviews

Currently, mdiff collects rich structured feedback (annotations with categories/severity, line scores, checklist items) but has no mechanism to bundle it all into a verdict. The prompt output is a flat dump with no decision metadata.

## Solution

Add an approve/reject workflow that:
1. Presents a decision dialog after the reviewer has examined the changeset
2. Bundles all feedback (annotations, scores, checklist status) with the verdict
3. Exports the complete review as structured data (clipboard + optional file)
4. Tracks the verdict in the session file for historical comparison

### User Flow

1. Reviewer finishes examining the diff, leaves annotations and scores
2. Press `A` (in DiffExplorer, not visual mode) to open the **Review Decision** dialog
3. Dialog shows a summary: annotation count, score distribution, checklist completion
4. Reviewer selects a verdict:
   - `a` — **Approve**: Agent output is acceptable, proceed
   - `r` — **Request Changes**: Agent needs to address feedback before proceeding
   - `x` — **Reject**: Agent output is fundamentally wrong, start over
   - `Esc` — Cancel (return to review without recording a verdict)
5. For Request Changes and Reject, reviewer can optionally add a summary comment
6. On confirm, the verdict is:
   - Saved to the session file
   - Included in the prompt output (prepended as a header)
   - Copied to clipboard automatically
   - Shown as a status badge in the HUD

### Verdict in Prompt Output

The existing prompt output (copied via `y`) gets enhanced:

```
## Review Verdict: REQUEST CHANGES

**Summary**: The implementation is mostly correct but has critical error handling gaps 
in the network layer. Tests are incomplete — only happy path covered.

**Stats**: 12 annotations (3 Critical, 5 Major, 4 Minor) | Avg score: 3.2/5 | 
Checklist: 4/6 complete

---

[existing annotation output follows]
```

## Architecture

### New State: `ReviewDecisionState` (src/state/review_decision_state.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewVerdict {
    Approve,
    RequestChanges,
    Reject,
}

impl ReviewVerdict {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Approve => "APPROVED",
            Self::RequestChanges => "REQUEST CHANGES",
            Self::Reject => "REJECTED",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            Self::Approve => 'a',
            Self::RequestChanges => 'r',
            Self::Reject => 'x',
        }
    }

    pub fn color(&self, theme: &Theme) -> Color {
        match self {
            Self::Approve => theme.success,        // Green
            Self::RequestChanges => theme.warning,  // Yellow
            Self::Reject => theme.error,            // Red
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReviewDecisionState {
    pub dialog_open: bool,
    pub verdict: Option<ReviewVerdict>,
    pub summary_comment: String,
    pub summary_editing: bool,  // Whether the summary text input is active
    pub recorded_at: Option<String>,  // ISO timestamp
}
```

### New Actions (src/action.rs)

```rust
// Review Decision
OpenReviewDecision,          // Open the verdict dialog
SelectVerdict(ReviewVerdict), // Select approve/request-changes/reject
ConfirmVerdict,              // Confirm and record the verdict
CancelReviewDecision,        // Close dialog without recording
VerdictSummaryChar(char),    // Type in summary comment
VerdictSummaryBackspace,
VerdictSummaryNewline,
StartVerdictSummary,         // Enter summary editing mode
```

### New Component: `ReviewDecisionDialog` (src/components/review_decision.rs)

Renders as a centered modal overlay (similar to commit dialog):

```
┌─────────────── Review Decision ───────────────┐
│                                                │
│  Annotations: 12 (3 Critical, 5 Major, 4 Minor)│
│  Avg Score: 3.2/5 (from 8 scored lines)       │
│  Checklist: 4/6 complete                       │
│                                                │
│  ─────────────────────────────────────────     │
│                                                │
│  [a] ● Approve    [r] ● Request Changes       │
│  [x] ● Reject     [Esc] Cancel                │
│                                                │
│  Summary (optional, press Enter to confirm):   │
│  > Error handling gaps in network layer_       │
│                                                │
└────────────────────────────────────────────────┘
```

The dialog:
- Shows aggregate stats from the current review session
- Highlights the selected verdict option
- Has an optional summary text field (activates after selecting a verdict)
- Pressing the verdict key selects it; pressing Enter confirms and closes

### Keybindings

- `A` (in DiffExplorer, not visual mode, not in other modals): Open review decision dialog
- Inside the dialog:
  - `a` — Select Approve
  - `r` — Select Request Changes
  - `x` — Select Reject
  - `Enter` — Confirm verdict (if one is selected). If summary editing is active, newline in summary.
  - `Esc` — Cancel / close dialog
  - When summary editing: standard text input keys

### Event Mapping (src/event.rs)

Add a new priority check before other modals:

```rust
// Priority 2.05: Review decision dialog
if ctx.review_decision_open {
    if ctx.verdict_summary_editing {
        // Text input mode for summary
        return match key.code {
            KeyCode::Esc => Some(Action::CancelReviewDecision),
            KeyCode::Enter => Some(Action::ConfirmVerdict),
            KeyCode::Backspace => Some(Action::VerdictSummaryBackspace),
            KeyCode::Char(c) => Some(Action::VerdictSummaryChar(c)),
            _ => None,
        };
    }
    return match key.code {
        KeyCode::Char('a') => Some(Action::SelectVerdict(ReviewVerdict::Approve)),
        KeyCode::Char('r') => Some(Action::SelectVerdict(ReviewVerdict::RequestChanges)),
        KeyCode::Char('x') => Some(Action::SelectVerdict(ReviewVerdict::Reject)),
        KeyCode::Enter => Some(Action::ConfirmVerdict),
        KeyCode::Esc => Some(Action::CancelReviewDecision),
        _ => None,
    };
}
```

### Integration with Prompt Output (src/app.rs)

Modify `render_prompt_for_all_files()` to prepend verdict header when a verdict is recorded:

```rust
fn render_prompt_for_all_files(&self) -> Option<String> {
    let mut output = String::new();
    
    // Prepend verdict if recorded
    if let Some(ref verdict) = self.state.review_decision.verdict {
        output.push_str(&format!("## Review Verdict: {}\n\n", verdict.label()));
        
        if !self.state.review_decision.summary_comment.is_empty() {
            output.push_str(&format!("**Summary**: {}\n\n", 
                self.state.review_decision.summary_comment));
        }
        
        // Add stats
        let stats = self.compute_review_stats();
        output.push_str(&format!("**Stats**: {} annotations | Avg score: {:.1}/5 | Checklist: {}\n\n",
            stats.annotation_summary,
            stats.avg_score,
            stats.checklist_summary,
        ));
        
        output.push_str("---\n\n");
    }
    
    // ... existing annotation rendering ...
}
```

### Session Persistence (src/session.rs)

Add verdict data to the session file:

```rust
#[derive(Serialize, Deserialize)]
struct SessionFile {
    version: u32,  // Bump to 4
    target_label: String,
    annotations: Vec<AnnotationEntry>,
    #[serde(default)]
    scores: Vec<ScoreEntry>,
    #[serde(default)]
    checklist: Option<ChecklistSessionData>,
    #[serde(default)]
    verdict: Option<VerdictSessionData>,
}

#[derive(Serialize, Deserialize)]
struct VerdictSessionData {
    verdict: String,  // "approve", "request_changes", "reject"
    summary: String,
    recorded_at: String,
}
```

### HUD Integration

When a verdict is recorded, show a badge in the status bar / HUD:

- Approve: `[APPROVED]` in green
- Request Changes: `[CHANGES REQUESTED]` in yellow  
- Reject: `[REJECTED]` in red

### Auto-Copy on Verdict

When the verdict is confirmed:
1. Generate the full prompt output (with verdict header)
2. Copy to clipboard automatically
3. Show status message: "Verdict recorded. Prompt copied to clipboard."
4. Save to session file

## AppState Integration

Add to `AppState` in `src/state/app_state.rs`:

```rust
pub struct AppState {
    // ... existing fields ...
    
    // Review decision
    pub review_decision: ReviewDecisionState,
}
```

Add to `KeyContext` in `src/event.rs`:

```rust
pub struct KeyContext {
    // ... existing fields ...
    pub review_decision_open: bool,
    pub verdict_summary_editing: bool,
}
```

## Edge Cases

- **No annotations or scores**: Verdict can still be recorded (sometimes a clean approve needs no comments)
- **Changing verdict**: If a verdict was already recorded, opening the dialog again shows the existing verdict and allows changing it
- **Session reload**: When loading a session, restore the verdict state so the HUD badge shows correctly
- **Multiple files**: Verdict applies to the entire review session, not per-file
- **Prompt output without verdict**: The existing `y` key still works for copying prompt without verdict. The verdict is only prepended when one has been recorded.

## Files to Modify

1. **src/state/review_decision_state.rs** (NEW) — ReviewVerdict enum, ReviewDecisionState struct
2. **src/state/mod.rs** — Export new module
3. **src/state/app_state.rs** — Add `review_decision` field to AppState, add KeyContext fields
4. **src/action.rs** — Add verdict-related action variants
5. **src/event.rs** — Add priority check for review decision dialog, map keybindings
6. **src/components/review_decision.rs** (NEW) — Dialog rendering
7. **src/components/mod.rs** — Export new component
8. **src/app.rs** — Handle verdict actions, integrate with prompt output, auto-copy on confirm
9. **src/session.rs** — Persist verdict in session file

## Testing

- Unit test: ReviewVerdict serialization/deserialization
- Unit test: Prompt output includes verdict header when recorded
- Unit test: Session persistence round-trip with verdict data
- Manual test: Open dialog, select verdict, confirm — verify clipboard content
- Manual test: Change verdict after initial recording
- Manual test: HUD badge appears with correct color
- Manual test: Dialog shows correct aggregate stats

## Acceptance Criteria

- `A` opens the review decision dialog from DiffExplorer
- Dialog shows annotation/score/checklist stats
- Three verdict options: Approve, Request Changes, Reject
- Optional summary comment for Request Changes and Reject
- Verdict is saved to session and included in prompt output
- Auto-copies structured prompt to clipboard on confirm
- HUD shows verdict badge with appropriate color
- Verdict persists across session save/load
