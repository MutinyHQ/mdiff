# Spec: Agent Guidance Export (Structured Feedback as Agent Instructions)

**Priority**: P1 (High Impact — Closes the Feedback Loop)
**Status**: Ready for implementation
**Estimated effort**: Medium (3-5 files changed)
**Inspiration**: Agent-RLVR (Scale Labs), CLAUDE.md ecosystem, GitHub Copilot PR comment consumption

## Problem

mdiff already exports annotations as JSON and markdown (spec 011, `Ctrl+E`). However, these exports are designed for *human* consumption — they describe what the reviewer observed but aren't formatted as actionable instructions that a coding agent can consume directly.

Research from Scale Labs' Agent-RLVR framework shows that structured feedback ("agent guidance") dramatically improves coding agent performance — from 9.4% to 22.4% on SWE-Bench. The key insight: diverse informational cues (strategic plans, error descriptions, environmental context) formatted as guidance signals are far more effective than raw observation data.

mdiff sits at the perfect intersection: reviewers produce structured annotations (with categories, severities, suggestions, and scores) that contain exactly the guidance signals agents need. The missing piece is an export format that agents can directly consume.

## Design

### Export Formats

Add three new export targets alongside the existing JSON/Markdown:

#### 1. CLAUDE.md Patch Format

Generate a patch for the project's `CLAUDE.md` (or `.cursorrules`, `AGENTS.md`, etc.) that encodes review feedback as persistent agent instructions:

```markdown
## Review Feedback (from mdiff review of <commit_sha>)

### Patterns to Avoid
- In `src/api/handler.rs`: Do not use unwrap() on user input parsing. Use proper error handling with `?` operator. (Severity: critical, Category: bug)
- In `src/db/queries.rs`: Avoid N+1 queries. Batch database operations when iterating over collections. (Severity: major, Category: performance)

### Patterns to Follow
- Error handling in `src/auth/` is well-structured. Continue using the `AppError` enum with `thiserror` derive.
- Test structure in `tests/integration/` follows good arrange-act-assert pattern. Maintain this.

### Specific Fixes Required
- `src/api/handler.rs:45-52`: Replace manual JSON parsing with serde derive. Suggested replacement provided.
- `src/db/migrations.rs:120`: Add index on `user_id` column for the new `sessions` table.
```

#### 2. Agent Instruction Format (Direct Prompt)

Generate a prompt that can be pasted directly into an agent (Claude Code, Codex, Cursor):

```
I reviewed your changes in <commit_range>. Here is my structured feedback:

## Critical Issues (must fix)
1. [src/api/handler.rs:45-52] BUG: unwrap() on user input will panic on malformed requests. Replace with proper error handling using the ? operator.
   Suggested fix:
   ```rust
   let input: UserInput = serde_json::from_str(&body)?;
   ```

2. [src/db/queries.rs:30-45] PERFORMANCE: N+1 query in user loading loop. Batch into a single WHERE IN query.

## Suggestions (should fix)
3. [src/api/handler.rs:80] STYLE: Function exceeds 50 lines. Extract validation logic into a separate function.

## Positive Observations (keep doing)
4. [src/auth/middleware.rs] Good error handling pattern with AppError enum.

Please address items 1-2 as critical fixes, then items 3 as improvements. Do not change the patterns noted in item 4.
```

#### 3. Git Trailer Format

Append structured feedback as git trailers that tools can parse:

```
Reviewed-by: <reviewer> via mdiff
Review-score: 3/5
Review-critical: src/api/handler.rs:45-52 unwrap on user input
Review-suggestion: src/db/queries.rs:30-45 batch N+1 query
Review-approved: src/auth/middleware.rs error handling pattern
```

### Implementation

#### New Export Module

Extend `src/export.rs` with new format generators:

```rust
pub enum ExportFormat {
    Json,           // Existing
    Markdown,       // Existing
    AgentGuidance,  // NEW: direct prompt for agents
    ClaudeMd,       // NEW: CLAUDE.md patch
    GitTrailers,    // NEW: git trailer format
}
```

#### Export Flow

1. User presses `Ctrl+E` to open export dialog (existing)
2. Export dialog now shows format picker: `[J]SON  [M]arkdown  [A]gent Guidance  [C]LAUDE.md  [G]it Trailers`
3. Selected format generates the output to clipboard and/or file
4. For Agent Guidance format, also offer "Copy to clipboard" for direct paste into agent

#### Annotation-to-Guidance Mapping

Map mdiff's annotation data to guidance signal types:

| mdiff Annotation | Agent Guidance Signal |
|------------------|----------------------|
| Category: bug, Severity: critical/major | "Critical Issues (must fix)" |
| Category: bug, Severity: minor | "Suggestions (should fix)" |
| Category: suggestion + suggested_code | "Specific Fixes Required" with code block |
| Category: style/nit | "Style Notes (low priority)" |
| Score: 4-5 (positive) | "Positive Observations (keep doing)" |
| Score: 1-2 (negative) | Maps to severity-appropriate issue category |
| Category: question | "Questions for Clarification" section |

#### Actions

```rust
// Modify existing export action to support format selection
ExportFeedbackWithFormat(ExportFormat),
// Or add a format picker step
OpenExportFormatPicker,
ExportFormatSelect(ExportFormat),
CancelExportFormatPicker,
```

### State Changes

```rust
// Add to AppState or create ExportState
pub struct ExportState {
    pub format_picker_open: bool,
    pub selected_format: ExportFormat,
    pub last_export_path: Option<PathBuf>,
}
```

## Implementation Steps

1. Add `ExportFormat` enum to `src/export.rs`
2. Add `ExportState` to `src/state/` and wire into `AppState`
3. Implement `generate_agent_guidance()` in `src/export.rs` — maps annotations to structured prompt
4. Implement `generate_claude_md_patch()` in `src/export.rs` — generates CLAUDE.md-compatible section
5. Implement `generate_git_trailers()` in `src/export.rs` — generates trailer-format output
6. Add format picker UI component in `src/components/export_picker.rs`
7. Update `src/event.rs` keybindings for format selection
8. Update `src/components/which_key.rs` with export format entries

## Edge Cases

- **No annotations**: Show "No annotations to export" message
- **Mixed severity annotations**: Group by severity section, then by file
- **Suggestions with code**: Include the suggested replacement code in agent guidance format as a fenced code block
- **Empty suggested_code**: Omit the "Suggested fix" sub-section
- **Very long exports**: For Agent Guidance format, warn if token count exceeds ~4000 tokens (rough estimate based on character count / 4)

## User Benefit

Transforms mdiff from a review *viewer* into a review *feedback system*. The reviewer's annotations don't just document observations — they become actionable instructions that directly improve the next agent run. This closes the human-in-the-loop cycle: agent produces code → reviewer annotates in mdiff → export generates agent guidance → agent consumes guidance → better code. Research shows this loop can improve agent success rates by 2-3x.
