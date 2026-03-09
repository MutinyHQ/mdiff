# Spec: Diff Complexity Indicators

**Priority**: P1
**Status**: Ready for implementation
**Estimated effort**: Medium (4-6 files changed)

## Problem

When reviewing a large coding agent changeset (20-50+ files), all hunks look equally important in the diff view. A one-line config change and a 40-line function rewrite with nested control flow get the same visual treatment. Reviewers waste time on trivial changes and risk rushing through complex ones. Research on AI code review effectiveness shows that high false-positive rates (treating everything as equally important) train developers to ignore the tool entirely.

Active learning research demonstrates that selecting the most informative data points for human review dramatically improves feedback quality. mdiff needs to surface visual signals that help reviewers prioritize their attention on the hunks that matter most.

## Solution

Add heuristic-based complexity indicators that appear in the diff view gutter and the file navigator, giving reviewers instant visual signal about which hunks and files need careful attention.

## Architecture

### New Module: `src/complexity.rs`

```rust
/// Complexity score for a hunk or file (0-10 scale).
#[derive(Debug, Clone, Copy)]
pub struct ComplexityScore {
    pub score: u8,          // 0-10
    pub label: &'static str, // "Low", "Med", "High", "Critical"
}

/// Hunk-level complexity analysis result.
#[derive(Debug, Clone)]
pub struct HunkComplexity {
    pub score: ComplexityScore,
    pub factors: Vec<ComplexityFactor>,
}

#[derive(Debug, Clone)]
pub enum ComplexityFactor {
    NestingDepthIncrease(u8),
    NewControlFlow(u8),      // if/match/loop/for/while added
    LargeHunkSize(usize),    // lines changed
    NewUnsafeBlock,
    NewUnwrapCall(u8),
    ErrorHandlingChange,
    PublicApiChange,
    NewDependency,
}

/// Analyze a hunk's added lines for complexity signals.
pub fn analyze_hunk(added_lines: &[&str], removed_lines: &[&str]) -> HunkComplexity { ... }

/// Aggregate hunk scores into a file-level score.
pub fn file_complexity(hunks: &[HunkComplexity]) -> ComplexityScore { ... }
```

### Heuristic Rules (no tree-sitter required)

The complexity analyzer uses simple text-pattern heuristics on added lines:

| Signal | Points | Detection |
|--------|--------|-----------|
| Nesting depth increase | +1 per level | Count leading indentation increase in added vs removed |
| New control flow | +1 each | Regex for `if `, `match `, `for `, `while `, `loop ` in added lines |
| Large hunk (>30 lines) | +2 | Line count |
| Very large hunk (>100 lines) | +4 | Line count |
| `unsafe` block added | +3 | Pattern match `unsafe {` or `unsafe fn` |
| `.unwrap()` added | +1 each | Pattern match `.unwrap()` in added lines not in removed |
| Error handling change | +1 | Pattern match `Result`, `Error`, `?` operator delta |
| Public API change | +2 | Pattern match `pub fn`, `pub struct`, `pub enum` in added |
| New dependency | +1 | File is `Cargo.toml` and line contains version specifier |

Score mapping:
- 0-2: Low (green)
- 3-4: Medium (yellow)
- 5-7: High (orange)
- 8+: Critical (red)

### UI Changes

#### Navigator (src/components/navigator.rs)

Add a colored complexity badge after each file name:
```
  src/main.rs          [Low]
  src/parser.rs        [High]
  src/auth/handler.rs  [Critical]
  Cargo.toml           [Med]
```

Badge colors match the score level. This gives instant triage signal in the file list.

#### Diff View Gutter (src/components/diff_view.rs)

At the start of each hunk header (the `@@` line), append a complexity badge:
```
@@ -45,12 +45,28 @@ fn process_request  [High: +control_flow, +nesting]
```

The badge shows the score label plus up to 2 top contributing factors.

### Integration with Existing Code

1. **DiffState** (src/state/diff_state.rs): Add `complexity_scores: HashMap<String, Vec<HunkComplexity>>` field, computed when a diff is loaded.

2. **Diff loading** (wherever `FileDelta` is populated): After parsing the diff, run `analyze_hunk` on each hunk's added/removed lines and store results.

3. **Navigator sorting**: When sort mode is active (#16 Smart Review Ordering), complexity scores can be used as a sort key.

4. **Which-key**: No new keybindings needed — complexity indicators are always visible. Could add `X` toggle to show/hide indicators if users find them noisy.

### New Actions (src/action.rs)

```rust
ToggleComplexityIndicators,  // Show/hide complexity badges (default: shown)
```

### Keybinding

- `X` (in DiffExplorer, not visual mode): Toggle complexity indicators on/off

### Configuration (config.toml)

```toml
[complexity]
enabled = true           # Default: true
min_display_score = 3    # Only show badges for Medium+ (hide Low)
```

## Edge Cases

- Binary files: Skip complexity analysis, show no badge
- Empty hunks (pure deletions): Analyze removed lines instead, label as "Removal" with lower weight
- Very large files (>1000 hunks): Cap analysis at first 200 hunks for performance
- Non-Rust files: Heuristics are language-agnostic (indentation, control flow keywords work across languages) but Rust-specific patterns (unsafe, unwrap) only apply to .rs files

## Testing

- Unit test: `analyze_hunk` with known inputs produces expected scores
- Unit test: Score aggregation at file level
- Unit test: Each heuristic rule triggers correctly
- Manual test: Large real-world agent changeset shows meaningful differentiation between files
