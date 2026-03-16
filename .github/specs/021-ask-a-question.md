# Spec: Ask a Question — Inline Q&A Over Diff Context (Issue #52)

**Priority**: P1 (High Impact — AI-Native Differentiator)
**Status**: Ready for implementation
**Estimated effort**: Medium-Large (6-8 files changed)
**Addresses**: GitHub Issue #52

## Problem

When reviewing a coding agent's changes, reviewers frequently encounter unfamiliar code, unclear intent, or complex logic they need to understand before providing meaningful feedback. Today, the reviewer must:

1. Leave mdiff to open a browser or terminal
2. Copy-paste code context into an LLM chat
3. Ask their question, wait for a response
4. Mentally map the answer back to the diff they were reviewing

This context-switching destroys flow state and adds friction to every "why?" moment during review. The reviewer loses their position in the diff, forgets which hunks they've already reviewed, and the feedback quality degrades.

## Competitive Reference

- **GitHub Copilot code review**: AI-generated line-level feedback, but initiated by the system, not the reviewer
- **Cursor IDE**: Inline AI chat with code context, the gold standard for editor-integrated Q&A
- **Aider**: Terminal-based AI coding assistant with chat interface
- **Duff**: Browser-based diff viewer — no Q&A capability
- **gstack `/review`**: AI-initiated review, not user-initiated questions

No existing TUI diff viewer offers inline Q&A. This is a clear differentiator for mdiff.

## Proposed Solution

### Interaction Flow

1. User presses `?` (or `Ctrl+Q`) while viewing a diff
2. A question input panel appears at the bottom of the diff view (similar to search bar)
3. User types their question (e.g., "Why was this function rewritten?")
4. On Enter, the question + diff context is sent to the configured LLM
5. The response streams into an answer panel overlaid on the diff view
6. User can dismiss with `Esc`, or navigate Q&A history with `Ctrl+]` / `Ctrl+[`
7. Previous Q&A pairs are stored in the session alongside annotations

### Context Assembly

When a question is submitted, assemble context from:

```rust
pub struct QuestionContext {
    pub file_path: String,
    pub file_language: Option<String>,  // inferred from extension
    pub visible_hunks: Vec<String>,     // hunks currently in the viewport
    pub selected_lines: Option<String>, // if visual selection is active
    pub full_diff: String,              // the complete file diff
    pub question: String,
}
```

**Context priority** (fit within model context window):
1. Selected lines (if visual mode was active before opening Q&A)
2. Current visible hunks in the viewport
3. Full file diff
4. File path and language hint

### New State: `QAState`

Add `src/state/qa_state.rs`:

```rust
use super::TextBuffer;

#[derive(Debug, Clone)]
pub struct QAPair {
    pub question: String,
    pub answer: String,
    pub file_path: String,
    pub line_context: Option<String>,  // the lines the question was about
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Default)]
pub struct QAState {
    pub input_open: bool,
    pub input_buffer: TextBuffer,
    pub answer_visible: bool,
    pub answer_text: String,
    pub answer_streaming: bool,       // true while response is being received
    pub history: Vec<QAPair>,
    pub history_index: Option<usize>, // when browsing previous Q&A pairs
    pub answer_scroll: usize,         // scroll offset for long answers
}
```

### New Actions

Add to `src/action.rs`:

```rust
// Q&A
OpenQuestionInput,
QuestionChar(char),
QuestionBackspace,
QuestionNewline,         // Shift+Enter for multiline questions
SubmitQuestion,          // Enter: send question to LLM
CancelQuestion,          // Esc: close input without sending
DismissAnswer,           // Esc when answer is showing: close answer panel
QAHistoryNext,           // Ctrl+]: next Q&A pair
QAHistoryPrev,           // Ctrl+[: previous Q&A pair
QAAnswerScrollUp,        // scroll long answers
QAAnswerScrollDown,
QAAnswerAppend(String),  // internal: append streamed text to answer
QAAnswerComplete,        // internal: streaming finished
```

### Key Mapping

In `src/event.rs`, add a new priority handler for Q&A state:

```rust
// Priority: Q&A input (when question input is open)
if ctx.qa_input_open {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('a') => Some(Action::TextCursorHome),
            KeyCode::Char('e') => Some(Action::TextCursorEnd),
            KeyCode::Char('w') => Some(Action::TextDeleteWord),
            _ => None,
        };
    }
    return match key.code {
        KeyCode::Esc => Some(Action::CancelQuestion),
        KeyCode::Enter => Some(Action::SubmitQuestion),
        KeyCode::Backspace => Some(Action::QuestionBackspace),
        KeyCode::Left => Some(Action::TextCursorLeft),
        KeyCode::Right => Some(Action::TextCursorRight),
        KeyCode::Char(c) => Some(Action::QuestionChar(c)),
        _ => None,
    };
}

// Priority: Q&A answer panel (when answer is visible)
if ctx.qa_answer_visible {
    return match key.code {
        KeyCode::Esc => Some(Action::DismissAnswer),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::QAAnswerScrollUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::QAAnswerScrollDown),
        _ => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char(']') => Some(Action::QAHistoryNext),
                    KeyCode::Char('[') => Some(Action::QAHistoryPrev),
                    _ => None,
                }
            } else {
                None
            }
        }
    };
}
```

Add the trigger in Priority 4 (global bindings, when diff view is focused):

```rust
KeyCode::Char('?') if !ctx.which_key_visible => {
    return Some(Action::OpenQuestionInput)
}
```

**Note**: `?` is currently used for `ToggleWhichKey`. This spec proposes reassigning `?` to Q&A (the more valuable interaction) and moving Which-Key to `F1` or keeping it at `?` only when the which-key overlay is already showing. Alternatively, use `Ctrl+Q` as the trigger. The agent implementing this should check the current `?` binding and choose the non-conflicting option.

### LLM Integration

Reuse the existing agent infrastructure from `src/state/agent_state.rs` and the `[[agents]]` configuration. The Q&A feature should:

1. Look for a configured agent (from `mdiff.toml` or default)
2. Build a prompt that includes the `QuestionContext` and the user's question
3. Send the request asynchronously (non-blocking — the TUI remains responsive)
4. Stream the response back via an `mpsc` channel, dispatching `QAAnswerAppend` actions

**Prompt template:**

```
You are a code review assistant. The user is reviewing a git diff and has a question about the changes.

File: {file_path}
Language: {language}

{diff_context}

{selected_lines_section}

User's question: {question}

Provide a concise, helpful answer. Focus on explaining the code changes and their intent. Keep the response under 500 words unless the question requires more detail.
```

If no agent is configured, show an error message: "No agent configured. Add an [[agents]] section to mdiff.toml to enable Q&A."

### UI Components

**Question Input Bar** (`src/components/qa_input.rs`):
- Renders at the bottom of the diff view area (similar to global search bar)
- Shows `? ` prompt followed by the text input with cursor
- Single line by default, wraps for long questions

```
┌─ diff_view ──────────────────────────────────┐
│ ... diff content ...                          │
│                                               │
├───────────────────────────────────────────────┤
│ ? Why was this error handling changed?█        │
└───────────────────────────────────────────────┘
```

**Answer Panel** (`src/components/qa_answer.rs`):
- Renders as a right-side panel or bottom panel within the diff view area
- Takes up ~40% of the diff view width (right panel) or ~40% height (bottom panel)
- Shows the question at the top, answer below with scrolling
- Streaming indicator `...` while response is being received
- Muted border with "Q&A" title

```
┌─ diff_view ─────────────────┬─ Q&A (1/3) ────────────┐
│ ... diff content ...         │ Q: Why was this error   │
│                              │    handling changed?    │
│                              │                         │
│                              │ A: The original code    │
│                              │ used unwrap() which     │
│                              │ would panic on invalid  │
│                              │ input. The new version  │
│                              │ uses proper Result      │
│                              │ propagation with the ?  │
│                              │ operator, making the    │
│                              │ function return an      │
│                              │ error instead of...     │
│                              │                         │
│                              │ [j/k] scroll [Esc] close│
│                              │ [Ctrl+]/[] history      │
└──────────────────────────────┴─────────────────────────┘
```

**History navigation**: Show `(1/3)` in the panel title when browsing history. `Ctrl+]` goes to the next (newer) pair, `Ctrl+[` to the previous (older) pair.

### Session Persistence

Q&A pairs should be saved alongside annotations in the session data:
- Add `qa_history: Vec<QAPair>` to the session serialization in `src/session.rs`
- Load previous Q&A pairs when reopening the same diff
- This allows reviewers to reference their previous questions across sessions

### KeyContext Update

Add to `KeyContext` in `event.rs`:

```rust
pub qa_input_open: bool,
pub qa_answer_visible: bool,
```

Populated from `state.qa.input_open` and `state.qa.answer_visible`.

## Files to Modify

1. **`src/state/qa_state.rs`** (NEW): `QAState`, `QAPair`, `QuestionContext`
2. **`src/state/mod.rs`**: Add `pub mod qa_state;`
3. **`src/state/app_state.rs`**: Add `qa: QAState` field to `AppState`
4. **`src/action.rs`**: Add Q&A actions
5. **`src/event.rs`**: Add `qa_input_open`, `qa_answer_visible` to `KeyContext`, add priority handlers, add `?` or `Ctrl+Q` trigger
6. **`src/components/qa_input.rs`** (NEW): Question input bar component
7. **`src/components/qa_answer.rs`** (NEW): Answer panel component with scrolling
8. **`src/components/mod.rs`**: Register new components
9. **`src/app.rs`**: Handle Q&A actions, manage LLM request/response lifecycle, render Q&A components
10. **`src/session.rs`**: Add Q&A history to session serialization/deserialization

## Testing

1. Press `?` or `Ctrl+Q` while viewing a diff — question input appears
2. Type a question and press Enter — answer panel appears with streaming response
3. Verify diff context is included (check the prompt sent to the agent)
4. Press `Esc` while answer is showing — answer panel closes
5. Press `?` again, ask another question — new Q&A pair added
6. Press `Ctrl+[` — navigate to previous Q&A pair
7. Press `Ctrl+]` — navigate to newer Q&A pair
8. Long answer: press `j`/`k` to scroll within answer panel
9. No agent configured: shows error message instead of hanging
10. Run `cargo check` — no compilation errors
11. Run `cargo test` — existing tests pass

## Edge Cases

- No agent configured: Display helpful error message with configuration instructions
- Very long question: Input wraps or scrolls horizontally
- Very long answer: Scrollable with `j`/`k` or arrow keys
- Network error / timeout: Show error in answer panel, allow retry
- Multiple rapid questions: Queue or cancel previous in-flight request
- Visual selection active when `?` pressed: Include selected lines as primary context
- Terminal resize while Q&A panel is open: Re-layout panels
- Empty question (just press Enter): Ignore, keep input open

## Backward Compatibility

If `?` is reassigned from Which-Key: move Which-Key to `F1` and note in the which-key overlay itself. Alternatively, use `Ctrl+Q` as the trigger to avoid any conflict. The implementing agent should check and choose the non-conflicting option.
