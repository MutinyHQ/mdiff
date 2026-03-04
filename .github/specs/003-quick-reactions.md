# Spec: Annotation Quick-Reactions (Single-Key Line Scoring)

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Medium (5-7 files changed)

## Problem

mdiff's full annotation flow (visual select → category picker → severity picker → comment editor → confirm) is powerful for detailed feedback but too heavy for rapid line-by-line quality assessment. When reviewing large agent-generated changesets (100+ changed lines across 20+ files), reviewers often have a quick gut reaction — "this line is good," "this looks wrong" — that they want to record without the overhead of a full annotation.

RLHF research shows that combining scalar ratings (numeric scores) with categorical feedback (text annotations) dramatically improves signal quality for model training. The scalar signal captures the reviewer's immediate quality judgment, while the categorical feedback explains *why*.

Currently mdiff has no way to capture this fast, low-friction feedback signal.

## Design

### Scoring System

| Key | Score | Color | Gutter Symbol | Meaning |
|-----|-------|-------|---------------|---------|
| `1` | 1/5 | Red | `●` | Bad — clearly wrong or harmful |
| `2` | 2/5 | Orange | `●` | Poor — needs significant rework |
| `3` | 3/5 | Yellow | `●` | Okay — acceptable but improvable |
| `4` | 4/5 | Light Green | `●` | Good — solid implementation |
| `5` | 5/5 | Green | `●` | Great — exemplary code |

### Behavior

- Press `1`-`5` on any line in the diff view (DiffView focus, no visual selection needed)
- The score attaches to the current cursor line (single line)
- If visual mode is active, the score attaches to the entire selection
- Score appears as a colored dot in the gutter (left of line numbers)
- Pressing a number on an already-scored line updates the score
- Pressing `0` on a scored line removes the score
- Scores are independent of annotations — a line can have both a score and a full annotation
- Scores persist in the session file alongside annotations

### Prompt Output

Scores are included in the generated prompt with a compact format:

```
### Scores
- src/main.rs:42 [Score: 1/5]
- src/main.rs:45-48 [Score: 4/5]
- src/lib.rs:12 [Score: 2/5]
```

When a line has both a score and an annotation, they combine:

```
### [Bug | Critical] Lines 45-48 [Score: 1/5]
The error handling here swallows the original error context...
```

### Status Bar

When the cursor is on a scored line, show the score in the status bar:
`Score: ●●●○○ (3/5)`

## Implementation

### 1. `src/state/annotation_state.rs` — Add score data structures

Add a new `LineScore` struct and scoring state:

```rust
/// A quick-reaction score attached to a line or range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineScore {
    pub file_path: String,
    pub old_range: Option<(u32, u32)>,
    pub new_range: Option<(u32, u32)>,
    pub score: u8, // 1-5
    pub created_at: String,
}

impl LineScore {
    /// Representative line for sorting.
    pub fn sort_line(&self) -> u32 {
        self.new_range
            .map(|(s, _)| s)
            .or(self.old_range.map(|(s, _)| s))
            .unwrap_or(0)
    }
}
```

Add to `AnnotationState`:

```rust
/// Quick-reaction scores (separate from annotations).
pub scores: BTreeMap<String, Vec<LineScore>>,
```

Add methods:

```rust
/// Set a score for a line/range. Replaces any existing score at the same position.
pub fn set_score(&mut self, score: LineScore) {
    let key = score.file_path.clone();
    let entries = self.scores.entry(key).or_default();
    // Remove any existing score at the same position
    entries.retain(|s| s.old_range != score.old_range || s.new_range != score.new_range);
    entries.push(score);
}

/// Remove score at a position.
pub fn remove_score(
    &mut self,
    file_path: &str,
    old_range: Option<(u32, u32)>,
    new_range: Option<(u32, u32)>,
) {
    if let Some(scores) = self.scores.get_mut(file_path) {
        scores.retain(|s| s.old_range != old_range || s.new_range != new_range);
        if scores.is_empty() {
            self.scores.remove(file_path);
        }
    }
}

/// Get score at a specific line position.
pub fn score_at(
    &self,
    file_path: &str,
    old_lineno: Option<u32>,
    new_lineno: Option<u32>,
) -> Option<u8> {
    self.scores.get(file_path).and_then(|scores| {
        scores.iter().find_map(|s| {
            let covers = match (new_lineno, s.new_range) {
                (Some(n), Some((start, end))) if n >= start && n <= end => true,
                _ => false,
            } || match (old_lineno, s.old_range) {
                (Some(n), Some((start, end))) if n >= start && n <= end => true,
                _ => false,
            };
            if covers { Some(s.score) } else { None }
        })
    })
}

/// Get all scores as a flat sorted list.
pub fn all_scores_sorted(&self) -> Vec<&LineScore> {
    let mut result: Vec<&LineScore> =
        self.scores.values().flat_map(|v| v.iter()).collect();
    result.sort_by_key(|s| (&s.file_path, s.sort_line()));
    result
}

/// Total score count.
pub fn score_count(&self) -> usize {
    self.scores.values().map(|v| v.len()).sum()
}
```

Initialize `scores` to `BTreeMap::new()` in `AnnotationState::default()`.

### 2. `src/action.rs` — Add score actions

```rust
// Quick-reaction scores
SetLineScore(u8),    // 1-5
RemoveLineScore,     // 0 key
```

### 3. `src/event.rs` — Add keybindings

In the `FocusPanel::DiffView` match block (Priority 8), add number key handling. These must NOT fire when any dialog/modal is open, which is already guaranteed by the priority system:

```rust
FocusPanel::DiffView => match key.code {
    // ... existing bindings ...
    KeyCode::Char('1') => Some(Action::SetLineScore(1)),
    KeyCode::Char('2') => Some(Action::SetLineScore(2)),
    KeyCode::Char('3') => Some(Action::SetLineScore(3)),
    KeyCode::Char('4') => Some(Action::SetLineScore(4)),
    KeyCode::Char('5') => Some(Action::SetLineScore(5)),
    KeyCode::Char('0') => Some(Action::RemoveLineScore),
    _ => None,
},
```

Also add score actions in visual mode (Priority 7):

```rust
if ctx.visual_mode_active && ctx.focus == FocusPanel::DiffView {
    return match key.code {
        // ... existing bindings ...
        KeyCode::Char('1') => Some(Action::SetLineScore(1)),
        KeyCode::Char('2') => Some(Action::SetLineScore(2)),
        KeyCode::Char('3') => Some(Action::SetLineScore(3)),
        KeyCode::Char('4') => Some(Action::SetLineScore(4)),
        KeyCode::Char('5') => Some(Action::SetLineScore(5)),
        KeyCode::Char('0') => Some(Action::RemoveLineScore),
        _ => None,
    };
}
```

### 4. `src/app.rs` — Handle score actions

```rust
Action::SetLineScore(value) => {
    // Determine the line range based on current cursor or visual selection
    // Follow the same pattern used by OpenCommentEditor to get file_path,
    // old_range, and new_range from the current cursor position or selection
    let (file_path, old_range, new_range) = /* extract from cursor/selection */;
    
    let score = LineScore {
        file_path: file_path.clone(),
        old_range,
        new_range,
        score: value,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    self.state.annotations.set_score(score);
    
    // If in visual mode, exit it after scoring
    if self.state.selection.active {
        self.state.selection.active = false;
    }
    
    let dots = "●".repeat(value as usize) + &"○".repeat(5 - value as usize);
    self.state.status_message = Some((
        format!("Score: {} ({}/5)", dots, value),
        false,
    ));
}

Action::RemoveLineScore => {
    let (file_path, old_range, new_range) = /* extract from cursor position */;
    self.state.annotations.remove_score(&file_path, old_range, new_range);
    self.state.status_message = Some(("Score removed".to_string(), false));
}
```

### 5. `src/components/diff_view.rs` — Render score indicators in gutter

In the gutter rendering logic, after annotation markers, add score dot rendering:

```rust
// Check for score at this line
if let Some(score) = state.annotations.score_at(file_path, old_lineno, new_lineno) {
    let color = match score {
        1 => Color::Red,
        2 => Color::Rgb(255, 165, 0), // Orange
        3 => Color::Yellow,
        4 => Color::Rgb(144, 238, 144), // Light green
        5 => Color::Green,
        _ => Color::Gray,
    };
    // Render a colored dot in the gutter area
    let dot = Span::styled("●", Style::default().fg(color));
    // Position in the gutter column (adjust based on existing gutter layout)
}
```

### 6. Prompt rendering — Include scores

In the prompt template rendering (wherever `render_prompt_for_all_files` or similar is implemented), add a scores section:

```rust
// Render scores section
let all_scores = self.state.annotations.all_scores_sorted();
if !all_scores.is_empty() {
    prompt.push_str("\n### Scores\n");
    for score in &all_scores {
        let range = match score.new_range {
            Some((s, e)) if s == e => format!("{}:{}", score.file_path, s),
            Some((s, e)) => format!("{}:{}-{}", score.file_path, s, e),
            None => match score.old_range {
                Some((s, e)) if s == e => format!("{}:{} (old)", score.file_path, s),
                Some((s, e)) => format!("{}:{}-{} (old)", score.file_path, s, e),
                None => score.file_path.clone(),
            },
        };
        prompt.push_str(&format!("- {} [Score: {}/5]\n", range, score.score));
    }
}
```

### 7. Session persistence

In the session save/load logic, include scores alongside annotations. The session file format should include:

```json
{
  "version": 2,
  "annotations": { ... },
  "scores": {
    "src/main.rs": [
      {
        "file_path": "src/main.rs",
        "old_range": null,
        "new_range": [42, 42],
        "score": 3,
        "created_at": "2026-03-05T10:00:00Z"
      }
    ]
  }
}
```

Use `#[serde(default)]` on the scores field for backward compatibility with existing sessions.

## Testing

- Open mdiff on a repo with changes
- Press `3` on a diff line — verify colored dot appears in gutter and status shows "Score: ●●●○○ (3/5)"
- Press `5` on the same line — verify dot changes to green and score updates
- Press `0` — verify dot removed and "Score removed" in status
- Enter visual mode, select 3 lines, press `2` — verify all lines get orange dots
- Open prompt preview — verify scores appear in output
- Save and reload session — verify scores persist
- Test that number keys do NOT fire in any modal/dialog (commit, comment, search, etc.)
