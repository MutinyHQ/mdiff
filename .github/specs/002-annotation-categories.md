# Spec: Annotation Categories & Severity Levels

**Priority**: P0
**Status**: Ready for implementation
**Estimated effort**: Medium (4-6 files changed)

## Problem

mdiff's annotation system currently supports free-text comments attached to line selections. When sending feedback to coding agents, structured annotations are far more effective than unstructured text. Research on RLHF and human feedback loops shows that categorized, severity-tagged feedback leads to better agent responses. This transforms mdiff from a "leave comments" tool into a structured feedback system.

The prompt template currently outputs flat comments. With categories and severity, the rendered prompt becomes machine-parseable and gives agents clear signal about what to prioritize.

## Design

### Annotation Categories

| Category | Key | Color | Description |
|----------|-----|-------|-------------|
| Bug | `b` | Red | Code is incorrect or produces wrong results |
| Style | `s` | Blue | Code style, formatting, naming conventions |
| Performance | `p` | Yellow | Inefficient code, unnecessary allocations |
| Security | `x` | Bright Red | Security vulnerabilities, unsafe patterns |
| Suggestion | `g` | Green | General improvement idea |
| Question | `q` | Cyan | Something needing clarification |
| Nitpick | `n` | Gray/Dim | Minor issue, not blocking |

### Severity Levels

| Severity | Key | Rendering |
|----------|-----|-----------|
| Critical | `c` | Bold + Red badge |
| Major | `M` | Yellow badge |
| Minor | `m` | Dim badge |
| Info | `i` | Gray badge |

### Updated Flow

**Before (current):**
`v` (select) → `i` (open comment editor) → type comment → `Enter`

**After:**
`v` (select) → `i` → **category picker** (single keypress) → **severity picker** (single keypress) → type comment → `Enter`

**Fast path:** Pressing `Enter` at the category picker skips to comment editor with defaults (Suggestion / Minor).

## Implementation

### 1. `src/state/annotation_state.rs` — Add category and severity types

Add before the `Annotation` struct:

```rust
/// Category of an annotation for structured agent feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationCategory {
    Bug,
    Style,
    Performance,
    Security,
    Suggestion,
    Question,
    Nitpick,
}

impl AnnotationCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Bug => "Bug",
            Self::Style => "Style",
            Self::Performance => "Perf",
            Self::Security => "Security",
            Self::Suggestion => "Suggestion",
            Self::Question => "Question",
            Self::Nitpick => "Nitpick",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            Self::Bug => 'b',
            Self::Style => 's',
            Self::Performance => 'p',
            Self::Security => 'x',
            Self::Suggestion => 'g',
            Self::Question => 'q',
            Self::Nitpick => 'n',
        }
    }

    pub fn all() -> &'static [AnnotationCategory] {
        &[
            Self::Bug,
            Self::Style,
            Self::Performance,
            Self::Security,
            Self::Suggestion,
            Self::Question,
            Self::Nitpick,
        ]
    }
}

/// Severity level of an annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

impl AnnotationSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Info => "Info",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            Self::Critical => 'c',
            Self::Major => 'M',
            Self::Minor => 'm',
            Self::Info => 'i',
        }
    }

    pub fn all() -> &'static [AnnotationSeverity] {
        &[Self::Critical, Self::Major, Self::Minor, Self::Info]
    }
}
```

Update the `Annotation` struct to include the new fields with serde defaults for backward compatibility:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub anchor: LineAnchor,
    pub comment: String,
    pub created_at: String,
    #[serde(default = "default_category")]
    pub category: AnnotationCategory,
    #[serde(default = "default_severity")]
    pub severity: AnnotationSeverity,
}

fn default_category() -> AnnotationCategory {
    AnnotationCategory::Suggestion
}

fn default_severity() -> AnnotationSeverity {
    AnnotationSeverity::Minor
}
```

### 2. `src/state/app_state.rs` — Add picker state

Add the phase enum and fields:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryPickerPhase {
    SelectCategory,
    SelectSeverity,
}
```

Add to `AppState`:

```rust
// Category/severity picker
pub category_picker_open: bool,
pub category_picker_phase: CategoryPickerPhase,
pub pending_category: Option<AnnotationCategory>,
pub pending_severity: Option<AnnotationSeverity>,
```

Initialize all to defaults (`false`, `CategoryPickerPhase::SelectCategory`, `None`, `None`).

Also update `AnnotationMenuItem` to include:

```rust
pub category: AnnotationCategory,
pub severity: AnnotationSeverity,
```

### 3. `src/action.rs` — Add picker actions

```rust
// Category/severity picker
OpenCategoryPicker,
SelectCategory(AnnotationCategory),
SelectSeverity(AnnotationSeverity),
CancelCategoryPicker,
CategoryPickerDefault, // Enter = skip to defaults (Suggestion/Minor)
```

### 4. `src/event.rs` — Add picker keybindings

Add a new priority block for the category picker (between comment_editor and settings modal):

```rust
// Priority 2.1: Category/severity picker
if ctx.category_picker_open {
    match ctx.category_picker_phase {
        CategoryPickerPhase::SelectCategory => {
            return match key.code {
                KeyCode::Char('b') => Some(Action::SelectCategory(AnnotationCategory::Bug)),
                KeyCode::Char('s') => Some(Action::SelectCategory(AnnotationCategory::Style)),
                KeyCode::Char('p') => Some(Action::SelectCategory(AnnotationCategory::Performance)),
                KeyCode::Char('x') => Some(Action::SelectCategory(AnnotationCategory::Security)),
                KeyCode::Char('g') => Some(Action::SelectCategory(AnnotationCategory::Suggestion)),
                KeyCode::Char('q') => Some(Action::SelectCategory(AnnotationCategory::Question)),
                KeyCode::Char('n') => Some(Action::SelectCategory(AnnotationCategory::Nitpick)),
                KeyCode::Enter => Some(Action::CategoryPickerDefault),
                KeyCode::Esc => Some(Action::CancelCategoryPicker),
                _ => None,
            };
        }
        CategoryPickerPhase::SelectSeverity => {
            return match key.code {
                KeyCode::Char('c') => Some(Action::SelectSeverity(AnnotationSeverity::Critical)),
                KeyCode::Char('M') => Some(Action::SelectSeverity(AnnotationSeverity::Major)),
                KeyCode::Char('m') => Some(Action::SelectSeverity(AnnotationSeverity::Minor)),
                KeyCode::Char('i') => Some(Action::SelectSeverity(AnnotationSeverity::Info)),
                KeyCode::Enter => Some(Action::SelectSeverity(AnnotationSeverity::Minor)),
                KeyCode::Esc => Some(Action::CancelCategoryPicker),
                _ => None,
            };
        }
    }
}
```

Add `category_picker_open` and `category_picker_phase` to the `KeyContext` struct.

### 5. `src/components/category_picker.rs` — New component

Create a new component that renders a compact bottom bar overlay:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::annotation_state::{AnnotationCategory, AnnotationSeverity};
use crate::state::app_state::CategoryPickerPhase;
use crate::state::AppState;

pub fn render_category_picker(frame: &mut Frame, area: Rect, state: &AppState) {
    if !state.category_picker_open {
        return;
    }

    let theme = &state.theme;

    // Render a centered overlay at the bottom of the screen
    let height = 3;
    let picker_area = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(height + 1),
        width: area.width.saturating_sub(2),
        height,
    };

    frame.render_widget(Clear, picker_area);

    let content = match state.category_picker_phase {
        CategoryPickerPhase::SelectCategory => {
            let items: Vec<Span> = AnnotationCategory::all()
                .iter()
                .flat_map(|cat| {
                    let color = category_color(cat, theme);
                    vec![
                        Span::styled(
                            format!("[{}]", cat.shortcut()),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{} ", cat.label()),
                            Style::default().fg(theme.text),
                        ),
                    ]
                })
                .collect();
            let title = " Category (Enter=Suggestion) ";
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));
            Paragraph::new(Line::from(items)).block(block)
        }
        CategoryPickerPhase::SelectSeverity => {
            let items: Vec<Span> = AnnotationSeverity::all()
                .iter()
                .flat_map(|sev| {
                    let color = severity_color(sev, theme);
                    vec![
                        Span::styled(
                            format!("[{}]", sev.shortcut()),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{} ", sev.label()),
                            Style::default().fg(theme.text),
                        ),
                    ]
                })
                .collect();
            let title = " Severity (Enter=Minor) ";
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));
            Paragraph::new(Line::from(items)).block(block)
        }
    };

    frame.render_widget(content, picker_area);
}

fn category_color(cat: &AnnotationCategory, theme: &crate::theme::Theme) -> Color {
    match cat {
        AnnotationCategory::Bug => theme.error,
        AnnotationCategory::Style => theme.accent,
        AnnotationCategory::Performance => theme.warning,
        AnnotationCategory::Security => Color::Rgb(255, 60, 60),
        AnnotationCategory::Suggestion => theme.success,
        AnnotationCategory::Question => Color::Cyan,
        AnnotationCategory::Nitpick => theme.text_muted,
    }
}

fn severity_color(sev: &AnnotationSeverity, theme: &crate::theme::Theme) -> Color {
    match sev {
        AnnotationSeverity::Critical => theme.error,
        AnnotationSeverity::Major => theme.warning,
        AnnotationSeverity::Minor => theme.text_muted,
        AnnotationSeverity::Info => theme.text_muted,
    }
}
```

### 6. `src/components/mod.rs` — Register new component

Add: `pub mod category_picker;`

### 7. `src/app.rs` — Wire up action handlers

**OpenCommentEditor flow change:** When the user presses `i` (which triggers `OpenCommentEditor`), instead of directly opening the comment editor, open the category picker first:

```rust
Action::OpenCommentEditor => {
    // Open category picker instead of comment editor directly
    self.state.category_picker_open = true;
    self.state.category_picker_phase = CategoryPickerPhase::SelectCategory;
    self.state.pending_category = None;
    self.state.pending_severity = None;
}

Action::SelectCategory(cat) => {
    self.state.pending_category = Some(cat);
    self.state.category_picker_phase = CategoryPickerPhase::SelectSeverity;
}

Action::SelectSeverity(sev) => {
    self.state.pending_severity = Some(sev);
    self.state.category_picker_open = false;
    // Now open the comment editor as before
    self.state.comment_editor_open = true;
    self.state.comment_editor_text = TextBuffer::new();
}

Action::CategoryPickerDefault => {
    self.state.pending_category = Some(AnnotationCategory::Suggestion);
    self.state.pending_severity = Some(AnnotationSeverity::Minor);
    self.state.category_picker_open = false;
    self.state.comment_editor_open = true;
    self.state.comment_editor_text = TextBuffer::new();
}

Action::CancelCategoryPicker => {
    self.state.category_picker_open = false;
    self.state.pending_category = None;
    self.state.pending_severity = None;
}
```

**ConfirmComment handler:** When saving the annotation, use the pending category and severity:

```rust
Action::ConfirmComment => {
    let category = self.state.pending_category.unwrap_or(AnnotationCategory::Suggestion);
    let severity = self.state.pending_severity.unwrap_or(AnnotationSeverity::Minor);
    // ... existing annotation creation logic ...
    // Set category and severity on the Annotation struct
    annotation.category = category;
    annotation.severity = severity;
    // ... rest of save logic ...
}
```

### 8. Prompt template update

In the `render_prompt_for_all_files` method (or wherever the prompt is rendered), update the annotation format to include structured metadata:

**Before:**
```
### Lines 45-52
The error handling here swallows the original error context...
```

**After:**
```
### [Bug | Critical] Lines 45-52
The error handling here swallows the original error context...
```

### 9. Render the picker in the main layout

In the main render function (wherever components are rendered to the frame), add:

```rust
category_picker::render_category_picker(frame, area, &self.state);
```

This should be rendered on top of other content (after the diff view).

## Backward Compatibility

- Existing annotation sessions (v2 format) will load correctly thanks to serde defaults — missing `category` defaults to `Suggestion`, missing `severity` defaults to `Minor`
- The session format version does not need to be bumped since the new fields are additive with defaults

## Testing

- Create an annotation: verify category picker appears first
- Select category with single keypress, verify severity picker appears
- Select severity, verify comment editor opens
- Press Enter at category picker — verify defaults to Suggestion/Minor
- Press Esc at any point — verify annotation is cancelled
- Save annotation and check prompt preview includes `[Category | Severity]` prefix
- Load old session file — verify annotations display as Suggestion/Minor
