# Spec: Agent Feedback Summary View

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Medium-Large (5-8 files changed)

## Problem

After reviewing an agent's changeset and leaving annotations and scores, the reviewer has no way to see the aggregate picture. Annotations are scattered across files, and there's no summary answering: "How many issues did I find? What categories? What's the overall quality?" This makes it hard to:

1. Assess overall agent performance before sending feedback
2. Compare agent quality across iterations
3. Communicate a high-level verdict to the agent alongside detailed annotations
4. Decide whether to accept, reject, or request revisions

The feedback loop is incomplete without a summary view that aggregates all structured feedback into an actionable overview.

## Design

### Toggle & Layout

- **Key**: `F` (in DiffExplorer view, not in visual mode)
- **Layout**: Full-screen overlay panel (similar to prompt preview), replacing the diff view
- **Exit**: `Esc` or `F` again to toggle back

### Summary Sections

#### Header
```
┌─────────────────────────────────────────────────┐
│  Feedback Summary — my-feature-branch            │
│  12 annotations · 34 scores · 8/20 files reviewed│
└─────────────────────────────────────────────────┘
```

#### Annotation Breakdown (by Category)
```
  Category        Count
  ─────────────────────
  Bug             ███░░░░░░░  3
  Style           █░░░░░░░░░  1
  Performance     ██░░░░░░░░  2
  Security        ░░░░░░░░░░  0
  Suggestion      ████░░░░░░  4
  Question        █░░░░░░░░░  1
  Nitpick         █░░░░░░░░░  1
```

#### Severity Distribution
```
  Critical  ██        2
  Major     ███       3
  Minor     █████     5
  Info      ██        2
```

#### Score Distribution (if quick-reactions are implemented)
```
  Score Distribution (34 lines scored)
  1 ●  ████████    8  (24%)
  2 ●  ██████      6  (18%)
  3 ●  ████████    8  (24%)
  4 ●  ██████████  10 (29%)
  5 ●  ██          2  (6%)
  
  Average: 2.8/5
```

#### Per-File Annotation Density
```
  File                          Annotations  Scores  Avg Score
  ─────────────────────────────────────────────────────────────
  src/main.rs                   4            12      2.3
  src/state/app_state.rs        3            8       3.1
  src/components/diff_view.rs   2            6       2.8
  src/event.rs                  2            5       3.5
  tests/integration.rs          1            3       4.2
```

#### Export Actions
```
  [y] Copy JSON to clipboard  [p] Copy as prompt text  [Esc] Close
```

### JSON Export Format

```json
{
  "session": "my-feature-branch",
  "timestamp": "2026-03-05T10:00:00Z",
  "summary": {
    "total_annotations": 12,
    "total_scores": 34,
    "files_reviewed": 8,
    "files_total": 20,
    "average_score": 2.8
  },
  "categories": {
    "Bug": 3,
    "Style": 1,
    "Performance": 2,
    "Security": 0,
    "Suggestion": 4,
    "Question": 1,
    "Nitpick": 1
  },
  "severities": {
    "Critical": 2,
    "Major": 3,
    "Minor": 5,
    "Info": 2
  },
  "score_distribution": {
    "1": 8,
    "2": 6,
    "3": 8,
    "4": 10,
    "5": 2
  },
  "files": [
    {
      "path": "src/main.rs",
      "annotations": 4,
      "scores": 12,
      "average_score": 2.3
    }
  ]
}
```

## Implementation

### 1. `src/state/app_state.rs` — Add feedback summary state

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    DiffExplorer,
    WorktreeBrowser,
    AgentOutputs,
    FeedbackSummary, // NEW
}
```

Add to `AppState`:

```rust
// Feedback summary
pub feedback_summary_scroll: usize,
```

Initialize `feedback_summary_scroll` to `0` in `AppState::new()`.

### 2. `src/action.rs` — Add actions

```rust
// Feedback summary
ToggleFeedbackSummary,
FeedbackSummaryUp,
FeedbackSummaryDown,
FeedbackSummaryCopyJson,
FeedbackSummaryCopyPrompt,
```

### 3. `src/event.rs` — Add keybindings

In Priority 6 (Diff explorer global bindings), add:

```rust
KeyCode::Char('F') => return Some(Action::ToggleFeedbackSummary),
```

Add a new priority block for the FeedbackSummary view (after AgentOutputs, Priority 5.5):

```rust
// Priority 5.6: Feedback summary view
if ctx.active_view == ActiveView::FeedbackSummary {
    return match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::FeedbackSummaryUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::FeedbackSummaryDown),
        KeyCode::Char('y') => Some(Action::FeedbackSummaryCopyJson),
        KeyCode::Char('p') => Some(Action::FeedbackSummaryCopyPrompt),
        KeyCode::Esc | KeyCode::Char('F') => Some(Action::ToggleFeedbackSummary),
        _ => None,
    };
}
```

Add `active_view` check in `KeyContext` (it's already there).

### 4. `src/app.rs` — Handle actions

```rust
Action::ToggleFeedbackSummary => {
    if self.state.active_view == ActiveView::FeedbackSummary {
        self.state.active_view = ActiveView::DiffExplorer;
    } else {
        self.state.active_view = ActiveView::FeedbackSummary;
        self.state.feedback_summary_scroll = 0;
    }
}

Action::FeedbackSummaryUp => {
    self.state.feedback_summary_scroll = self.state.feedback_summary_scroll.saturating_sub(1);
}

Action::FeedbackSummaryDown => {
    self.state.feedback_summary_scroll += 1;
}

Action::FeedbackSummaryCopyJson => {
    let json = self.build_feedback_summary_json();
    if let Ok(text) = serde_json::to_string_pretty(&json) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(&text);
            self.state.status_message = Some(("Feedback JSON copied to clipboard".to_string(), false));
        }
    }
}

Action::FeedbackSummaryCopyPrompt => {
    let prompt_text = self.build_feedback_summary_prompt();
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(&prompt_text);
        self.state.status_message = Some(("Feedback summary copied to clipboard".to_string(), false));
    }
}
```

Add helper methods to the `App` impl:

```rust
fn build_feedback_summary_json(&self) -> serde_json::Value {
    let annotations = &self.state.annotations;
    
    // Count by category (requires annotation categories from spec 002)
    // If categories are not yet implemented, use a simpler count
    let total_annotations = annotations.count();
    let total_scores = annotations.score_count();
    let files_with_annotations: std::collections::HashSet<&str> = 
        annotations.annotations.keys().map(|s| s.as_str()).collect();
    let total_files = self.state.navigator.file_count(); // or however files are counted
    
    // Score distribution
    let all_scores = annotations.all_scores_sorted();
    let mut score_dist = [0usize; 5];
    let mut score_sum = 0usize;
    for s in &all_scores {
        if s.score >= 1 && s.score <= 5 {
            score_dist[(s.score - 1) as usize] += 1;
            score_sum += s.score as usize;
        }
    }
    let avg_score = if total_scores > 0 {
        score_sum as f64 / total_scores as f64
    } else {
        0.0
    };
    
    serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "total_annotations": total_annotations,
            "total_scores": total_scores,
            "files_with_feedback": files_with_annotations.len(),
            "files_total": total_files,
            "average_score": (avg_score * 10.0).round() / 10.0,
        },
        "score_distribution": {
            "1": score_dist[0],
            "2": score_dist[1],
            "3": score_dist[2],
            "4": score_dist[3],
            "5": score_dist[4],
        }
    })
}

fn build_feedback_summary_prompt(&self) -> String {
    let json = self.build_feedback_summary_json();
    let total = json["summary"]["total_annotations"].as_u64().unwrap_or(0);
    let scores = json["summary"]["total_scores"].as_u64().unwrap_or(0);
    let avg = json["summary"]["average_score"].as_f64().unwrap_or(0.0);
    
    format!(
        "## Review Summary\n\n\
         - {} annotations across {} files\n\
         - {} lines scored, average quality: {:.1}/5\n\n\
         See detailed annotations in the full prompt output.",
        total,
        json["summary"]["files_with_feedback"],
        scores,
        avg,
    )
}
```

### 5. `src/components/feedback_summary.rs` — New component

Create a new component for rendering the feedback summary view:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;
use super::Component;

pub struct FeedbackSummary;

impl Component for FeedbackSummary {
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let block = Block::default()
            .title(" Feedback Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state.theme.accent));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();

        // Header
        let total_ann = state.annotations.count();
        let total_scores = state.annotations.score_count();
        let reviewed = state.review.reviewed_count();
        let total_files = state.navigator.file_count();

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} annotations", total_ann),
                Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" · "),
            Span::styled(
                format!("{} scores", total_scores),
                Style::default().fg(state.theme.accent),
            ),
            Span::raw(" · "),
            Span::styled(
                format!("{}/{} files reviewed", reviewed, total_files),
                Style::default().fg(state.theme.text_muted),
            ),
        ]));
        lines.push(Line::from(""));

        // Score distribution
        if total_scores > 0 {
            lines.push(Line::from(Span::styled(
                "  Score Distribution",
                Style::default().add_modifier(Modifier::BOLD).fg(state.theme.text),
            )));
            lines.push(Line::from(""));

            let all_scores = state.annotations.all_scores_sorted();
            let mut dist = [0usize; 5];
            let mut sum = 0usize;
            for s in &all_scores {
                if s.score >= 1 && s.score <= 5 {
                    dist[(s.score - 1) as usize] += 1;
                    sum += s.score as usize;
                }
            }

            let max_count = *dist.iter().max().unwrap_or(&1).max(&1);
            let bar_width = 20usize;
            let colors = [Color::Red, Color::Rgb(255, 165, 0), Color::Yellow, Color::Rgb(144, 238, 144), Color::Green];

            for (i, count) in dist.iter().enumerate() {
                let filled = if max_count > 0 { count * bar_width / max_count } else { 0 };
                let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                let pct = if total_scores > 0 { *count * 100 / total_scores } else { 0 };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", i + 1), Style::default().fg(colors[i])),
                    Span::styled("● ", Style::default().fg(colors[i])),
                    Span::styled(bar, Style::default().fg(colors[i])),
                    Span::styled(
                        format!("  {} ({}%)", count, pct),
                        Style::default().fg(state.theme.text_muted),
                    ),
                ]));
            }

            let avg = if total_scores > 0 { sum as f64 / total_scores as f64 } else { 0.0 };
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  Average: {:.1}/5", avg),
                Style::default().add_modifier(Modifier::BOLD).fg(state.theme.text),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Footer with actions
        lines.push(Line::from(vec![
            Span::styled("  [y]", Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" Copy JSON  "),
            Span::styled("[p]", Style::default().fg(state.theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" Copy prompt text  "),
            Span::styled("[Esc]", Style::default().fg(state.theme.text_muted)),
            Span::raw(" Close"),
        ]));

        // Apply scroll offset
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(state.feedback_summary_scroll)
            .collect();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }
}
```

### 6. `src/components/mod.rs` — Register component

Add: `pub mod feedback_summary;`

### 7. Main render function — Route to summary view

In the main render logic, when `state.active_view == ActiveView::FeedbackSummary`, render the feedback summary component instead of the diff explorer:

```rust
match state.active_view {
    ActiveView::DiffExplorer => { /* existing render */ }
    ActiveView::WorktreeBrowser => { /* existing render */ }
    ActiveView::AgentOutputs => { /* existing render */ }
    ActiveView::FeedbackSummary => {
        feedback_summary::FeedbackSummary.render(frame, main_area, &state);
    }
}
```

## Dependencies

- **Soft dependency on spec 002** (annotation categories): The category/severity breakdown section requires categories to be implemented. If not yet available, show a simpler "Total annotations: N" without category breakdown.
- **Soft dependency on spec 003** (quick-reactions): The score distribution section requires scores. If not yet available, omit the score section.

Both dependencies are handled gracefully — the summary view works with whatever data is available.

## Testing

- Open mdiff with some annotations (and scores if available)
- Press `F` — verify summary view appears with correct counts
- Verify scroll works with `j`/`k`
- Press `y` — verify JSON copied to clipboard with correct structure
- Press `p` — verify prompt text copied
- Press `Esc` — verify returns to diff explorer
- Press `F` again — verify toggles back to summary
- Test with zero annotations — verify empty state renders cleanly
- Test with annotations but no scores — verify score section is omitted
