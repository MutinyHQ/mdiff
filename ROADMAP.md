# mdiff Roadmap & Feature Backlog

> **Maintained by**: Milo (automated ideation agent)
> **Last updated**: 2026-03-07
> **Schedule**: New ideas are evaluated and prioritized every 24 hours

This document tracks feature ideas, prioritized issues, and the rationale behind them. It draws from competitive analysis of tools like **critique**, **tuicr**, **acre**, **difi**, **deff**, **git-review**, **Kaleidoscope**, **lazygit**, **fzf**, **justshowmediff**, **IPE**, **diffreview**, and patterns from RLHF/human feedback research.

---

## Priority Tiers

### P0 — Critical Path (Active PRs / Bugs)

#### 1. Hunk-Level Navigation (`]` / `[` keys)
**Status**: Merged
**Rationale**: Every serious diff tool (vim-fugitive, lazygit, tig, delta) supports jumping between hunks. Currently mdiff only has line-by-line (`j`/`k`) and page scrolling. For large agent-generated diffs with hundreds of lines, reviewers need to skip context and jump between actual changes.
**Scope**: Add `JumpNextHunk` / `JumpPrevHunk` actions, remap existing `]`/`[` annotation navigation to `Ctrl+]`/`Ctrl+[`, display "Hunk N/M" in the HUD, support cross-file hunk jumping.
**Competitive reference**: vim `[c`/`]c`, lazygit hunk navigation, tuicr `{`/`}` keys.

#### 2. Annotation Categories & Severity Levels
**Status**: PR #12 open (draft, targets main — needs retarget to develop)
**Rationale**: Structured feedback is dramatically more effective than free-text when communicating with coding agents. Research on RLHF shows categorized, severity-tagged annotations lead to better model responses. This transforms mdiff from a "leave comments" tool into a structured feedback system.
**Scope**: Add category picker (Bug/Style/Performance/Security/Suggestion/Question/Nitpick) and severity levels (Critical/Major/Minor/Info) to the annotation flow. Single-keypress selection. Update prompt template to include structured metadata. Backward-compatible with existing sessions.
**Known issue**: Session persistence hardcodes category=Suggestion, severity=Minor for loaded annotations. Needs AnnotationEntry format update.
**Competitive reference**: GitHub PR review comment types, V7 Labs RLHF annotation categories, Linear issue priority levels.

#### 3. Global Fuzzy Search Across All Diff Content (`Ctrl+F`)
**Status**: PR #13 open (draft)
**Rationale**: The existing `/` search only filters file names in the navigator. When reviewing large agent changesets spanning 20+ files, you need to find specific code patterns, variable names, or string literals across ALL changed files. The `nucleo` crate is already in Cargo.toml but only used for file name filtering.
**Scope**: Add `Ctrl+F` for global content search across all diff hunks, using nucleo for fuzzy matching with an exact-match toggle (`Ctrl+G`). Live incremental results, match highlighting in diff view, "Match N/M" counter, cross-file navigation with `n`/`N`.
**Competitive reference**: fzf search patterns, VS Code Ctrl+Shift+F, Kaleidoscope full-text search.

#### 4. Fix Diff Line Calculations (Issue #25) — NEW
**Status**: Spec written (009), Cursor agent launched
**Addresses**: GitHub Issue #25
**Rationale**: When using `G` (ScrollToBottom) in the diff view, the viewport does not render the full diff. Lines below the viewport boundary are inaccessible. This is a **critical usability bug** — reviewers cannot see all changes, which directly undermines the core mission. Filed by repo owner.
**Scope**: Audit scroll bound calculation vs display map row count, ensure consistent row counting across build_display_map/scroll clamping/cursor bounds, handle edge cases (hunk headers, gap lines, line wrapping). Fix in both Split and Unified views.
**Root cause**: Likely mismatch between logical line count used for scroll bounds and actual visual row count after hunk headers and gap lines.

---

### P1 — High Impact

#### 5. File Tree Navigator with Directory Grouping
**Rationale**: When agents modify 30+ files across multiple directories, the flat file list becomes unwieldy. A collapsible tree view grouped by directory (like VS Code's explorer) would make navigation much faster.
**Scope**: Add tree view mode to the navigator (toggle with `T`), collapse/expand directories, show file counts per directory, sort by path or by change type.
**Competitive reference**: VS Code file explorer, GitHub PR file tree, lazygit file tree.

#### 6. Diff Statistics Dashboard
**Rationale**: Before diving into line-by-line review, reviewers need an overview: how many files changed, total additions/deletions, which files have the most churn. This is especially important for agent output where you want to quickly assess if the agent went off track.
**Scope**: Add a summary view (toggle with `S`) showing: total files/additions/deletions, per-file sparkline bars, file type breakdown, largest files by change size, annotation coverage stats.
**Competitive reference**: GitHub PR stats bar, `git diff --stat`, diffray summary.

#### 7. Configurable Keybinding System
**Rationale**: Power users expect to customize keybindings. The current hardcoded mapping in `event.rs` works but doesn't allow personalization. A config-driven system would let users remap any key and create custom workflows.
**Scope**: Add `[keybindings]` section to `~/.config/mdiff/config.toml`, support conditional bindings (different behavior per context), vim-mode awareness, document all bindable actions.
**Competitive reference**: Atuin's conditional keybinding system, lazygit custom keybindings, neovim keymaps.

#### 8. Review Progress Tracking with Completion Percentage
**Rationale**: The existing review state tracks per-file reviewed/unreviewed, but doesn't show overall progress. For large reviews, knowing "I've reviewed 12/34 files (35%)" with a progress bar is essential for staying motivated and tracking sessions.
**Scope**: Add progress bar to the HUD, percentage complete, time tracking per file, estimated remaining time, ability to export review summary.
**Competitive reference**: tuicr review tracking, GitHub PR review progress, Reviewable progress bars.

#### 9. Annotation Quick-Reactions (Single-Key Line Scoring)
**Status**: PR #14 open (draft, targets main — needs retarget to develop)
**Rationale**: Full annotation flow (select -> category -> severity -> comment) is powerful but heavy for rapid feedback. RLHF research shows that scalar ratings alongside categorical feedback dramatically improves signal quality. Quick-reactions let reviewers score individual lines or hunks with a single keypress (1-5 scale) without entering any text, enabling rapid per-line quality scoring for agent feedback loops.
**Scope**: Press `1`-`5` on any diff line (or visual selection) to leave a quick score. Scores render as colored dots in the gutter (red->green gradient). Included in prompt output as `[Score: 3/5]` prefix. Dashboard shows score distribution. Compatible with full annotations (score + comment). Persist in session file alongside annotations.
**Competitive reference**: RLHF scalar reward signals, Likert scales in annotation tools, GitHub emoji reactions.

#### 10. Agent Feedback Summary View
**Status**: PR #15 open (draft)
**Rationale**: After leaving annotations and scores, reviewers need to see the aggregate picture before sending feedback to agents. A summary view answers: "What did I flag? How severe was it? Is this agent improving?" Currently annotations are scattered across files with no aggregate view. This closes the feedback loop.
**Scope**: Toggle with `F` -- dedicated panel showing: annotation count by category/severity, score distribution histogram, per-file annotation density, one-click structured JSON export for agent consumption, trend comparison across sessions (if multiple sessions exist for the same worktree).
**Competitive reference**: GitHub PR review summary, RLHF annotation dashboards, Linear project analytics.

#### 11. Contextual Help Overlay (Which-Key)
**Status**: Merged (PR #16)
**Rationale**: mdiff has 50+ keybindings across 8+ contexts (navigator, diff view, visual mode, comment editor, etc.). Discovery is a major UX challenge -- the HUD toggle (`?`) shows a static list, but power users need contextual hints. Lazygit, neovim (which-key), and Helix all show available keys for the current context. Every new feature we add increases the discoverability problem.
**Scope**: After a brief idle period (300ms) with no keypress following a "leader" key, show a floating overlay listing available follow-up keys and their actions for the current context. Covers all focus panels and modal states. Dismisses on any keypress. Can be disabled in config. Shows key + short description in a compact grid.
**Competitive reference**: neovim which-key.nvim, Helix context help, Emacs which-key, lazygit contextual hints.

#### 12. Review Checklist Templates
**Status**: PR #21 open (draft, targets develop) — Cursor agent FINISHED
**Rationale**: When reviewing coding agent output, reviewers often need to check the same things every time: error handling, test coverage, no hardcoded secrets, proper logging. Currently mdiff collects freeform annotations and scores, but there's no structured way to ensure a reviewer has covered all the important dimensions. RLHF research shows that checklists dramatically improve annotation consistency and reduce the "forgetting to check" failure mode.
**Scope**: Configurable per-project checklists loaded from config.toml. Toggleable panel (`C`) showing items with single-keypress toggling. Checklist completion included in prompt output to agents. Persisted in session files. Progress indicator in HUD.
**Competitive reference**: GitHub PR review checklists, RLHF annotation quality control, surgical safety checklists adapted for code review.

#### 13. Diff Complexity Indicators
**Status**: PR #22 open (draft, targets develop) — Cursor agent FINISHED
**Rationale**: In large agent changesets, all hunks look equally important. A one-line config change and a 40-line function rewrite with nested control flow get the same visual treatment. Active learning research shows that surfacing the most informative items for human review dramatically improves feedback quality. High false-positive rates (treating everything as equally important) train developers to ignore the tool.
**Scope**: Heuristic-based complexity scoring (0-10) per hunk and file. Colored badges in navigator (`[Low]`, `[High]`, `[Critical]`) and diff view gutter. Factors: nesting depth, control flow additions, unsafe blocks, unwrap calls, public API changes, hunk size. Toggle with `X`. No tree-sitter required -- uses text-pattern heuristics.
**Competitive reference**: Active learning for annotation prioritization, SonarQube complexity metrics, GitHub code scanning severity levels.

#### 14. Word-Level (Intra-Line) Diff Highlighting — NEW (promoted from P2)
**Status**: Spec written (010), Cursor agent launched
**Rationale**: Currently the diff shows entire lines as added/deleted. For modified lines, highlighting the specific words or characters that changed within a line makes review dramatically faster. Every major competitor (critique, GitHub, delta, VS Code) has this. It is table-stakes UX for a diff viewer in 2026. Without it, mdiff forces reviewers to visually scan every modified line character-by-character.
**Scope**: Compute word-level diffs for paired deletion/addition lines using the `similar` crate. Highlight changed segments with brighter/more saturated variants of the base add/delete colors. Toggle with `Ctrl+W`. Pair detection: consecutive deletions followed by equal-count additions. Cap computation at 500 chars per line. Works in both Split and Unified views.
**Competitive reference**: critique word-level diffs, GitHub word-level highlighting, delta intra-line highlights, VS Code character-level diff.

#### 15. Approve/Reject Agent Workflow — NEW
**Status**: Spec written (011), Cursor agent launched
**Rationale**: After reviewing an agent's changeset and leaving annotations/scores, there is no explicit decision point. Research on IPE (a Claude Code review interface) and GitHub's PR review model shows that an explicit approve/reject verdict dramatically improves the feedback loop: it forces a deliberate judgment, structures exported feedback with a clear verdict, enables tracking outcomes over time, and creates a natural end-of-review ritual.
**Scope**: Review decision dialog (`A` key) showing annotation/score/checklist stats. Three verdicts: Approve, Request Changes, Reject. Optional summary comment. Verdict prepended to prompt output, auto-copied to clipboard, saved in session, shown as HUD badge.
**Competitive reference**: GitHub PR approve/request-changes, IPE approve/reject workflow, code review decision systems.

#### 16. Fix cmd+K Kill Wrong Session (Issue #24) — NEW
**Status**: Open issue, needs investigation
**Addresses**: GitHub Issue #24
**Rationale**: When multiple agent sessions exist in the output pane, Cmd+K kills the wrong session instead of the focused one. This is a UX bug that causes confusion and potential data loss in the agent workflow.
**Scope**: Investigate routing of KillAgentProcess action in agent_outputs component. Ensure the kill targets the currently focused/selected session, not a hardcoded or wrong index.

---

### P2 — Medium Impact

#### 17. Inline AI Suggestions (Agent-in-the-Loop)
**Rationale**: Instead of just sending feedback TO agents, mdiff could also receive suggestions FROM agents during review. Imagine selecting a problematic section, pressing a key, and getting an inline AI suggestion for how to fix it -- without leaving the diff view.
**Scope**: Add `Ctrl+S` to request an AI suggestion for the selected code, display inline suggestion in a overlay panel, accept/reject/modify the suggestion, integrate with configured agents.

#### 18. Multi-Worktree Comparison View
**Rationale**: When multiple agents work on the same problem in parallel worktrees, you want to compare their approaches side by side. Currently you can only view one worktree at a time.
**Scope**: Add a comparison mode that shows two worktree diffs side by side, highlight differences between agent approaches, allow cherry-picking hunks from either side.

#### 19. Bookmark System for Large Reviews
**Rationale**: In multi-session reviews spanning hours, you need to bookmark positions to return to later. Annotations serve a different purpose (feedback for agents). Bookmarks are for the reviewer's own navigation.
**Scope**: Add `b` to toggle a bookmark on the current line, `B` to open bookmark list, bookmarks persist per session, show bookmark indicators in the gutter.

#### 20. Git Blame Integration
**Rationale**: When reviewing agent changes, it's useful to see the blame context -- who wrote the code being modified and when. This helps assess risk and understand the codebase better.
**Scope**: Add `B` to toggle blame view on the old side, show author/date/commit for each line, click to expand blame details.

#### 21. Smart Review Ordering (Auto-Sort Files by Priority)
**Rationale**: Agent changesets often touch 20-50+ files spanning tests, implementation, config, and docs. Reviewing in alphabetical or git order is suboptimal. Research on code review effectiveness shows that reviewing tests first (to understand intent), then core implementation, then peripheral files leads to better bug detection. Flat file lists bury the most important files.
**Scope**: Add sort modes toggled with `O`: by change size (largest first), by file type priority (tests -> impl -> config -> docs), by annotation density, by review status. Show sort indicator in navigator header. Remember sort preference in config. Can integrate with complexity indicators (#13) for content-aware sorting.
**Competitive reference**: GitHub PR file ordering, Reviewable smart ordering, IntelliJ change list grouping.

#### 22. Glob-Based File Filtering
**Rationale**: The current `/` search does fuzzy matching on file names, which works for finding a specific file. But when reviewing agent output touching `*.rs`, `*.toml`, `*.md`, and `*.yaml` files, you often want to filter to just one type or exclude test files. fzf-lua's glob filtering in live grep is a beloved power-user feature for exactly this pattern.
**Scope**: Extend the navigator search (`/`) to support glob patterns when input starts with `>` (e.g., `>*.rs`, `>!*test*`, `>src/components/**`). Show "glob mode" indicator. Combine with existing fuzzy search (glob narrows, then fuzzy matches within). Leverage nucleo for performance.
**Competitive reference**: fzf-lua glob filtering, VS Code file exclude patterns, ripgrep glob flags.

#### 23. Diff Heatmap Mode (Change Density Visualization)
**Rationale**: In the navigator, all files look the same regardless of how much they changed. In the diff view, it's hard to see where changes cluster in a large file. A heatmap provides instant visual signal about where to focus attention -- critical when triaging large agent changesets.
**Scope**: Color-code files in the navigator by change density (red=heavy, yellow=moderate, green=minor). Add a scrollbar minimap in the diff view showing change density along the file height. Toggle with `H`. Colors configurable in theme.
**Competitive reference**: VS Code minimap change indicators, Sublime Text minimap, GitHub contribution heatmap.

#### 24. Mouse Support for Navigation
**Status**: Cursor agent EXPIRED — needs re-implementation
**Rationale**: mdiff captures mouse events but does not process them. While keyboard-driven workflow is core, many developers use mouse as secondary input. Competitors like lazygit support mouse alongside keyboard shortcuts. Scroll wheel support is especially useful for browsing diffs naturally.
**Scope**: Scroll wheel in diff view and navigator, click-to-select files in navigator, click-to-focus panels. Navigation only -- annotation/editing remain keyboard-driven. Mouse can be disabled in config. Requires storing layout rects from render pass for hit-testing.
**Competitive reference**: lazygit mouse support, VS Code terminal mouse, tig mouse scrolling.

#### 25. Watch Mode with Auto-Refresh
**Rationale**: critique has watch mode that auto-refreshes when files change. mdiff has manual refresh (`R`). When an agent is actively modifying files in a worktree, auto-detecting changes and refreshing the diff would create a smoother workflow.
**Scope**: Add file watcher (notify crate) on the active worktree. Auto-refresh diff when changes detected, with debounce (500ms). Show "watching..." indicator in HUD. Toggle with `W`. Preserve scroll position and review state across refreshes.
**Competitive reference**: critique watch mode, nodemon-style file watching, VS Code live reload.

#### 26. Syntax-Aware Folding
**Rationale**: In large diffs, unchanged context lines between hunks take up significant screen space. Collapsing unchanged function bodies or blocks would let reviewers focus on actual changes, similar to VS Code's code folding.
**Scope**: Press `z` to fold/unfold the current context block. Fold indicators in gutter. Fold all/unfold all with `Z`. Would benefit from tree-sitter for accurate block detection but can start with indentation-based heuristics.
**Competitive reference**: VS Code code folding, neovim fold, GitHub PR file collapsing.

#### 27. Diff Snapshot / Time Travel
**Rationale**: When an agent iterates on feedback across multiple rounds, reviewers want to compare what changed between iterations. Currently each review session is independent with no way to see the delta between agent attempts.
**Scope**: Save diff snapshots with timestamps. Compare any two snapshots to see what the agent changed between iterations. "Iteration 1 had 5 bugs flagged, iteration 2 has 2 -- show me what was fixed." Builds on session persistence infrastructure.
**Competitive reference**: GitHub PR commit history, Reviewable iteration tracking, Google Docs version history.

---

### P3 — Nice to Have

#### 28. Session Export to Markdown/JSON
Export the complete review session (annotations, diff stats, review progress) to a structured format for sharing or archival.

#### 29. Notification on Agent Completion
Watch for agent process completion in worktrees and show a notification in the TUI.

#### 30. Custom Prompt Templates
Allow users to define custom prompt template formats in config.toml for different agent types.

#### 31. Image Diff Support
For changes to image files, show a visual comparison (pixel diff, side-by-side, onion skin).

#### 32. Split Pane for Multiple Files
View two different files' diffs simultaneously in a horizontal or vertical split.

#### 33. Undo/Redo for Git Operations
Add undo capability for stage/unstage/restore operations to prevent accidental data loss.

#### 34. Diff Filter by Change Type
Filter the file list to show only additions, deletions, modifications, or renames.

#### 35. Integration with GitHub/GitLab PR Workflows
Push annotations as PR review comments directly from mdiff.

#### 36. Command Transparency Log
**Rationale**: Lazygit's most-loved community feature is showing the underlying git commands it runs. This builds trust, helps users learn git, and aids debugging. mdiff performs git operations (stage, unstage, restore, commit) but the user never sees what's happening underneath.
**Scope**: Floating log panel toggled with `~` showing recent git commands and their stdout/stderr output. Scrollable history. Commands timestamped.

#### 37. External Editor Integration for Long Annotations
**Rationale**: The TUI text input is fine for short comments but limiting for detailed architectural feedback. The standard Unix pattern (used by git commit, crontab -e, etc.) is to open `$EDITOR` for long-form text.
**Scope**: Press `E` in the comment editor to open the current annotation text in `$EDITOR`. On editor close, import the text back. Supports multi-paragraph feedback with proper formatting.

---

## Open Issues Triage (2026-03-07)

| Issue | Title | Type | Priority | Action |
|-------|-------|------|----------|--------|
| #25 | Diff line calculations are off | Bug | **P0** | Promoted to roadmap #4. Spec 009 written. Cursor agent launched. |
| #24 | cmd+K in agents tab kills wrong session | Bug | **P1** | Added to roadmap #16. Needs investigation of agent outputs routing. |

Both open issues are filed by the repo owner (alechoey) and represent real usability problems. Issue #25 is critical — it prevents seeing all diff content. Issue #24 is a workflow bug in the agent integration that causes confusion.

---

## Competitive Landscape Notes

### Tools Analyzed
- **critique** (313+ stars): TypeScript/Bun TUI, syntax highlighting, split view, word-level diffs, watch mode. Strong UX but no agent integration.
- **tuicr**: Rust TUI for AI-generated diff review, GitHub PR-style infinite scroll, vim keybindings, clipboard export as structured Markdown. Closest competitor to mdiff's vision.
- **acre**: Python TUI for AI-assisted collaborative code review with Claude. Real-time collaboration model is interesting but different use case.
- **difi**: Go TUI with Neovim integration, recently posted on Hacker News. Community feedback emphasized importance of high-quality screenshots/GIFs for TUI tool adoption.
- **deff**: Rust TUI for side-by-side git diff review, posted on HN March 2026. Very early (8 commits). Has per-file navigation, syntax highlighting, vim motions. No annotation or agent features -- validates the space but not a competitive threat.
- **git-review**: Rust TUI for interactive code review, very early stage. Uses Claude AI integration. Direct Rust-peer competitor.
- **justshowmediff** (NEW 2026-03-05): Zero-dependency Go tool generating self-contained HTML diff viewers. Explicitly designed for Claude Code and Codex headless agent workflows. Supports piped stdin, staged/unstaged changes, branch comparisons. Validates that agent-oriented diff viewing is a growing market.
- **IPE** (NEW 2026-03-04): Intercepts Claude Code's ExitPlanMode hook to provide a GitHub-style code review interface with inline comments, file reference popovers, plan version comparison, and approve/request-changes workflow. Directly inspired the Approve/Reject feature (#15).
- **diffreview** (NEW 2026-03): Zsh helpers piping Git diff ranges into Claude Code or GitHub Copilot CLI for AI-powered code review, commit messages, PR descriptions, and release notes.
- **Kaleidoscope**: macOS-native diff/merge tool. Excellent visual polish but no TUI, no agent integration.
- **lazygit**: Gold standard for TUI git UX. Hunk-level operations, keyboard-driven workflow. Community's most-loved feature: showing underlying git commands.
- **fzf**: Gold standard for fuzzy search UX in terminals. Event-action binding, scoring algorithms.
- **jjui**: TUI for jujutsu (jj), panel-based keyboard-driven paradigm for commit graph editing. Shows the lazygit UX pattern has legs beyond git.

### Key Market Gap
No tool combines all three: (1) TUI-native diff review, (2) structured human feedback collection, (3) direct integration with coding agents. mdiff is uniquely positioned here.

### HN Signal (2026-03-05)
A commenter on the Deff HN thread explicitly requested a TUI diff tool with the ability to "comment on lines/ranges in a diff to provide targeted feedback to coding agents." This validates mdiff's core value proposition and suggests growing demand for agent-feedback-oriented review tools.

### UX Patterns Worth Adopting (from 2026-03-05 research)
- **Contextual keybindings per panel** (lazygit, jjui): Different keys do different things depending on which panel has focus. mdiff already does this but should formalize it.
- **Glob-based filtering in search** (fzf-lua): Users love being able to narrow results with glob patterns before fuzzy matching.
- **Command transparency** (lazygit): Showing the underlying commands builds trust and helps learning.
- **Disable dangerous operations by default** (lazygit community feedback): Force-push disabled, restore requires confirmation (mdiff already has this).

### RLHF / Active Learning Insights (from 2026-03-06 research)
- Modern alignment has evolved: DPO, GRPO, and RLVR are reducing reliance on traditional reward models. Structured annotation schemas should capture multiple signal dimensions.
- Active learning principles (selecting the most informative data points for human labeling) directly apply to deciding which hunks need review. This informed the Diff Complexity Indicators feature (#13).
- High false-positive rates in AI code review cause reviewers to disengage entirely. Complexity indicators help avoid the "everything looks important" failure mode.

### Agent Feedback Patterns (from 2026-03-07 research)
- **Structured inline annotation + verdict**: IPE demonstrates that explicit approve/reject decisions improve agent feedback loops. mdiff now implements this with the Approve/Reject workflow (#15).
- **Machine-readable feedback export**: justshowmediff and diffreview show the trend toward structured, tool-consumable feedback formats. mdiff's prompt output should evolve toward JSON alongside Markdown.
- **Review-as-a-quality-gate**: The 2026 consensus is that human code review is the single most important quality gate between AI agent output and production. Scoring, checklists, and verdicts all formalize this gate.

---

## Changelog

### 2026-03-07
- **PROMOTED Issue #25 to P0** (#4): Diff line calculations bug — spec 009, Cursor agent launched
- **ADDED P1 #14**: Word-Level Diff Highlighting (promoted from P2 #16 due to competitive pressure) — spec 010, Cursor agent launched
- **ADDED P1 #15**: Approve/Reject Agent Workflow (new, inspired by IPE research) — spec 011, Cursor agent launched
- **ADDED P1 #16**: Fix cmd+K Kill Wrong Session (Issue #24) — needs investigation
- Added justshowmediff, IPE, diffreview to competitive landscape
- Added "Agent Feedback Patterns" section to research notes
- Updated PR statuses: #21 (Checklist) and #22 (Complexity) Cursor agents FINISHED
- Noted Mouse Support agent (#24) EXPIRED — needs re-implementation
- Renumbered P2/P3 items to accommodate new P1 additions
- Open Issues Triage table added

### 2026-03-06
- Added P1 items #11 (Review Checklist Templates), #12 (Diff Complexity Indicators)
- Added P2 items #21 (Mouse Support), #22 (Watch Mode), #23 (Syntax-Aware Folding), #24 (Diff Snapshot/Time Travel)
- Updated statuses: #1 Hunk Navigation -> Merged, #10 Which-Key -> Merged
- Added Deff to competitive landscape (new Rust TUI diff tool on HN)
- Added HN Signal section noting explicit demand for agent feedback in diff tools
- Added RLHF / Active Learning Insights section
- Renumbered P2/P3 items to accommodate new additions
- Created specs 006 (Review Checklist), 007 (Complexity Indicators), 008 (Mouse Support)

### 2026-03-05
- Added P1 items #8 (Quick-Reactions), #9 (Feedback Summary View), #10 (Which-Key)
- Added P2 items #16 (Smart Review Ordering), #17 (Glob File Filtering), #18 (Diff Heatmap)
- Added P3 items #27 (Command Transparency Log), #28 (External Editor Integration)
- Updated competitive landscape with difi, git-review, jjui, and lazygit UX research
- Added "UX Patterns Worth Adopting" section
- Renumbered items to accommodate new additions
- Created specs for #8, #9, #10 (003, 004, 005)

### 2026-03-04
- Initial roadmap created
- Opened PRs for P0 items #1, #2, #3
- Competitive analysis of critique, tuicr, acre, Kaleidoscope, lazygit, fzf
- Researched RLHF/human feedback patterns for annotation design
