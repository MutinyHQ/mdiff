# Spec: Word-Level (Intra-Line) Diff Highlighting

**Priority**: P1 (promoted from P2)
**Status**: Ready for implementation
**Estimated effort**: Medium (3-5 files changed)
**Roadmap item**: #16

## Problem

Currently, mdiff highlights entire lines as added (green) or deleted (red). When a line is modified (appears as a deletion followed by an addition), the reviewer must visually scan the entire line to find what actually changed. For long lines or subtle changes (e.g., a single character rename, a type change, an operator swap), this is slow and error-prone.

Every major competitor now has word-level diff highlighting: critique has it, GitHub has it, delta has it, VS Code has it. This is table-stakes UX for a diff viewer in 2026. Without it, mdiff forces reviewers to do unnecessary cognitive work on every modified line.

## Solution

Compute word-level (intra-line) diffs for paired deletion/addition lines within hunks, then highlight the changed segments with a brighter/distinct color within the already-colored line background.

### Algorithm

For each hunk, identify paired lines: consecutive deletion(s) followed by consecutive addition(s) of equal count. For each pair:

1. **Tokenize** both lines into words (split on whitespace and punctuation boundaries)
2. **Compute LCS** (Longest Common Subsequence) or use a Myers diff on the token sequences
3. **Map changed tokens** back to character offsets in the original strings
4. **Emit highlight spans** for the changed regions

For simplicity and performance, use a character-level diff (not token-level) via the `similar` crate's `TextDiff` with character-level granularity. The `similar` crate is lightweight, pure Rust, and commonly used for exactly this purpose.

### Pairing Strategy

Within each hunk, scan for "change groups": consecutive deletions immediately followed by consecutive additions.

```rust
/// A paired change: old lines removed, new lines added.
struct ChangePair {
    old_lines: Vec<(usize, String)>,  // (hunk_line_index, content)
    new_lines: Vec<(usize, String)>,  // (hunk_line_index, content)
}
```

When the number of old and new lines match, pair them 1:1. When they don't match (e.g., 2 deletions + 3 additions), fall back to no intra-line highlighting for that group — it's likely a structural change, not a modification.

### Highlight Representation

Add a new field to the rendering data:

```rust
/// Intra-line change spans for a single diff line.
#[derive(Debug, Clone)]
pub struct IntraLineSpan {
    pub start: usize,  // byte offset in content string
    pub end: usize,    // byte offset (exclusive)
}

/// Per-line intra-line change data, keyed by (file_path, display_row).
pub struct IntraLineHighlights {
    /// Map from display_row -> Vec<IntraLineSpan>
    highlights: HashMap<usize, Vec<IntraLineSpan>>,
}
```

### Rendering

In `diff_view.rs`, when rendering an addition or deletion line that has intra-line highlights:

- **Base style**: Keep the existing line background (green-tinted for additions, red-tinted for deletions)
- **Changed spans**: Apply a brighter/more saturated variant of the base color
  - Deletions: normal bg `#3a2030` → changed span bg `#6a2040` (brighter red)
  - Additions: normal bg `#203a30` → changed span bg `#206a40` (brighter green)
- **Unchanged spans within the line**: Keep the base style

This creates a two-level visual hierarchy: line-level change direction (add/delete) + word-level change location.

## Architecture

### New Module: `src/intra_diff.rs`

```rust
use similar::{ChangeTag, TextDiff};

/// Compute intra-line change spans for a pair of old/new lines.
pub fn compute_intra_line_spans(
    old_line: &str,
    new_line: &str,
) -> (Vec<IntraLineSpan>, Vec<IntraLineSpan>) {
    let diff = TextDiff::from_chars(old_line, new_line);
    
    let mut old_spans = Vec::new();
    let mut new_spans = Vec::new();
    let mut old_offset = 0usize;
    let mut new_offset = 0usize;
    
    for change in diff.iter_all_changes() {
        let len = change.value().len();
        match change.tag() {
            ChangeTag::Equal => {
                old_offset += len;
                new_offset += len;
            }
            ChangeTag::Delete => {
                old_spans.push(IntraLineSpan {
                    start: old_offset,
                    end: old_offset + len,
                });
                old_offset += len;
            }
            ChangeTag::Insert => {
                new_spans.push(IntraLineSpan {
                    start: new_offset,
                    end: new_offset + len,
                });
                new_offset += len;
            }
        }
    }
    
    (old_spans, new_spans)
}

/// Scan a hunk's lines and compute intra-line highlights for all change pairs.
pub fn compute_hunk_intra_highlights(
    lines: &[DiffLine],
) -> HashMap<usize, Vec<IntraLineSpan>> {
    // 1. Identify change groups (consecutive deletions followed by additions)
    // 2. For equal-count groups, pair 1:1 and compute spans
    // 3. Return map from line index -> spans
}
```

### New Dependency: `similar` crate

Add to Cargo.toml:
```toml
similar = "2"
```

The `similar` crate is a well-maintained, pure-Rust diff library with 1.5k+ stars. It provides character-level, word-level, and line-level diffing. We use `TextDiff::from_chars()` for maximum granularity.

### Integration with DiffState

Store computed intra-line highlights alongside the diff data:

```rust
// In src/state/diff_state.rs
pub struct DiffState {
    // ... existing fields ...
    
    /// Intra-line highlight spans per file, keyed by file path.
    /// Inner map: display_row -> Vec<IntraLineSpan>
    pub intra_highlights: HashMap<String, HashMap<usize, Vec<IntraLineSpan>>>,
    
    /// Whether intra-line highlighting is enabled (default: true)
    pub intra_line_enabled: bool,
}
```

### Computation Trigger

Compute intra-line highlights when diff data is loaded/refreshed (in the same place complexity analysis runs in app.rs):

```rust
// After deltas are loaded
if self.state.diff.intra_line_enabled {
    self.compute_intra_line_highlights();
}
```

### Rendering Integration (src/components/diff_view.rs)

In `build_split_lines_core` and `build_unified_lines_core`, when rendering addition/deletion content lines:

1. Check if intra-line spans exist for this display row
2. If yes, split the content string at span boundaries
3. Apply the "changed" style to spans within the highlight ranges
4. Apply the base style to spans outside

```rust
fn apply_intra_line_highlights(
    content: &str,
    spans: &[IntraLineSpan],
    base_style: Style,
    highlight_style: Style,
    syntax_spans: &[HighlightSpan],
) -> Vec<Span<'_>> {
    // Merge syntax highlighting with intra-line highlighting
    // Intra-line bg takes priority, syntax fg is preserved
}
```

### Toggle Keybinding

- Press `Ctrl+W` (mnemonic: **W**ord diff) in DiffExplorer to toggle intra-line highlighting
- Default: enabled
- Show status in HUD: "Word: On/Off"

### New Actions (src/action.rs)

```rust
ToggleIntraLineDiff,  // Toggle word-level highlighting
```

## Edge Cases

- **Very long lines** (>500 chars): Cap the diff computation to avoid performance issues. If either line exceeds 500 chars, skip intra-line highlighting for that pair.
- **Binary content or non-UTF8**: Skip gracefully
- **Entire line changed**: If the diff shows the entire line as changed (no common subsequence), don't highlight — the whole-line coloring is sufficient
- **Tabs and whitespace**: Include in diff computation. Whitespace-only changes should still be highlighted (they're often significant).
- **Unicode**: The `similar` crate handles Unicode correctly via char-level diffing
- **Performance**: Cache results in `intra_highlights` map. Only recompute on diff refresh, not on scroll.

## Theme Integration

Add new theme colors:

```rust
pub struct Theme {
    // ... existing colors ...
    pub diff_add_word_bg: Color,     // Brighter green for changed words in additions
    pub diff_delete_word_bg: Color,  // Brighter red for changed words in deletions
}
```

## Files to Modify

1. **Cargo.toml** — Add `similar = "2"` dependency
2. **src/intra_diff.rs** (NEW) — Core intra-line diff computation
3. **src/main.rs** — Add `mod intra_diff;`
4. **src/state/diff_state.rs** — Add `intra_highlights` and `intra_line_enabled` fields
5. **src/app.rs** — Trigger computation on diff load, handle `ToggleIntraLineDiff` action
6. **src/components/diff_view.rs** — Integrate highlights into line rendering for both split and unified views
7. **src/action.rs** — Add `ToggleIntraLineDiff` variant
8. **src/event.rs** — Map `Ctrl+W` to the toggle action
9. **src/theme.rs** — Add word-level highlight colors

## Testing

- Unit test: `compute_intra_line_spans` with simple character changes
- Unit test: `compute_intra_line_spans` with word additions/deletions
- Unit test: Pairing logic with equal and unequal change group sizes
- Unit test: Edge case with empty lines, very long lines, unicode
- Manual test: View a diff with renamed variables — changed chars should be highlighted
- Manual test: Toggle with `Ctrl+W` — highlights appear/disappear
- Manual test: Both split and unified views show highlights correctly
- Manual test: Performance with large diffs (1000+ lines) — should not lag

## Acceptance Criteria

- Modified lines show word-level highlights in both split and unified views
- Changed characters/words are visually distinct from unchanged portions of the same line
- Syntax highlighting is preserved (fg from syntax, bg from intra-line)
- Toggle with `Ctrl+W` works
- No performance regression on large diffs
- Graceful fallback for unpaired or very long lines
