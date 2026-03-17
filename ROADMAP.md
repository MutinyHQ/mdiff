# mdiff Roadmap & Feature Backlog

> **Maintained by**: Milo (automated ideation agent)
> **Last updated**: 2026-03-17
> **Schedule**: New ideas are evaluated and prioritized every 24 hours

This document tracks feature ideas, prioritized issues, and the rationale behind them. It draws from competitive analysis of tools like **critique**, **tuicr**, **acre**, **difi**, **deff**, **git-review**, **Kaleidoscope**, **lazygit**, **fzf**, **justshowmediff**, **IPE**, **diffreview**, **Fresh**, **1Code**, **Duff**, **patchcast**, and patterns from RLHF/human feedback research.

---

## Priority Tiers

### P0 — Critical Path (Active PRs / Bugs)

#### 1. Hunk-Level Navigation (`]` / `[` keys)
**Status**: Merged

#### 2. Annotation Categories & Severity Levels
**Status**: Merged

#### 3. Global Fuzzy Search Across All Diff Content (`Ctrl+F`)
**Status**: Merged (core feature). **UX Bug fixed** — Issue #35, PR #51 merged (2026-03-13). Navigation moved to Ctrl+N/Ctrl+P.

#### 4. Fix Diff Line Calculations (Issue #25)
**Status**: **Fixed** — PR #50 merged (2026-03-14). Added `clamp_scroll()` function to prevent invalid scroll offsets. Issue should be closed.
**Addresses**: GitHub Issue #25

#### 5. Remove `q` Keybinding for Quit (Issue #38)
**Status**: **Closed** — Issue #38 resolved.
**Addresses**: GitHub Issue #38

---

### P1 — High Impact

#### 5. File Tree Navigator with Directory Grouping (Issue #53)
**Status**: Spec written (020), Cursor agent blocked (model unavailable 2026-03-16, 2026-03-17)
**Addresses**: GitHub Issue #53
**Rationale**: When agents modify 30+ files across multiple directories, the flat file list becomes unwieldy. A collapsible tree view grouped by directory would make navigation much faster. Filed by repo owner with detailed acceptance criteria.
**Scope**: Add tree view mode to the navigator (toggle with `T`), collapse/expand directories with Enter, show file counts per directory, box-drawing indent guides, zM/zR for fold all/unfold all.
**Competitive reference**: VS Code file explorer, GitHub PR file tree, lazygit file tree, Yazi TUI file manager.

#### 6. Diff Statistics Dashboard
**Status**: Spec written (019), Cursor agent blocked (model unavailable 2026-03-13, 2026-03-16, 2026-03-17)
**Rationale**: Before diving into line-by-line review, reviewers need an overview: how many files changed, total additions/deletions, which files have the most churn. No way to quickly assess changeset scope without scrolling through every file.
**Scope**: Add a summary view (toggle with `S`) showing: total files/additions/deletions, per-file sparkline bars, file type breakdown, largest files by change size. Jump-to-file from the dashboard.
**Competitive reference**: GitHub PR stats bar, `git diff --stat`, diffray summary.

#### Ask a Question: Inline Q&A Over Diff Context (Issue #52)
**Status**: Spec written (021), Cursor agent blocked (model unavailable 2026-03-16, 2026-03-17)
**Addresses**: GitHub Issue #52
**Rationale**: Reviewers frequently encounter unfamiliar code or unclear intent during review and must leave the TUI to ask an LLM. This destroys flow state. Inline Q&A with diff context assembly eliminates the context switch entirely. No other TUI diff viewer offers this — a clear differentiator. Filed by repo owner.
**Scope**: `Ctrl+Q` trigger, question input bar at bottom of diff view, context assembly from visible hunks/selection, answer panel as right-side split, Q&A history with Ctrl+]/Ctrl+[, session persistence.
**Competitive reference**: Cursor IDE inline chat, Aider terminal AI assistant. No TUI diff viewer competitor has this.

#### 7. Configurable Keybinding System
**Rationale**: Power users expect to customize their keybindings. As the action set grows, conflicts become more likely.
**Scope**: TOML config file for key remapping, runtime reload.

#### 8. Annotation Quick-Reactions (Single-Key Line Scoring)
**Status**: Merged
**Spec**: 003

#### 9. Agent Feedback Summary View
**Status**: Merged
**Spec**: 004

#### 10. Contextual Help Overlay (Which-Key)
**Status**: Merged
**Spec**: 005

#### 11. Review Checklist Templates
**Status**: Spec written (006), Cursor agent finished
**Spec**: 006

#### 12. Diff Complexity Indicators
**Status**: Spec written (007), Cursor agent finished
**Spec**: 007

#### 13. Structured Feedback Export (`Ctrl+E`)
**Status**: Spec written (011), Cursor agent finished
**Spec**: 011

#### 14. Word-Level Diff Highlighting
**Status**: Spec written (010), Cursor agent finished
**Spec**: 010

#### 15. Approve/Reject Agent Workflow
**Status**: Spec written (011-approve-reject), Cursor agent finished
**Spec**: 011-approve-reject-workflow

#### 16. Fix cmd+K Kill Wrong Session (Issue #24)
**Status**: Merged (PR #46)

#### Command Palette (`Ctrl+P`)
**Status**: Spec written (018), Cursor agent pending
**Rationale**: mdiff has grown to 50+ distinct actions. The which-key overlay shows keybindings for the current context, but users must memorize bindings or scan a static list. A fuzzy-searchable command palette (inspired by VS Code, Fresh editor) would make all actions discoverable in 2-3 keystrokes. As the feature count grows, this becomes critical for onboarding and power-user efficiency.
**Scope**: Add `Ctrl+P` trigger, floating panel with fuzzy search input (using nucleo crate), register ~30-40 user-facing actions with labels and keybinding hints, dispatch selected action through normal handle_action flow.
**Competitive reference**: VS Code `Ctrl+Shift+P`, Fresh editor `Ctrl+P`, Neovim Telescope command picker.

#### Review Progress Bar
**Status**: NEW — promoted from P2 (2026-03-17)
**Rationale**: Prevents missed files in large changesets. Show per-file and overall review completion percentage. Track files viewed, annotated, and marked reviewed. Visual progress bar in navigator header. Motivated by patchcast's scene-based sequential review pattern and the growing size of agent changesets. As mdiff adds more features (tree navigator, stats dashboard), users need a persistent visual signal of how much review remains.
**Scope**: Track review state per file (unviewed / viewed / annotated / marked reviewed). Show compact progress bar `[====----] 4/8 reviewed` in navigator header. Update automatically as user navigates and interacts. Persist across sessions.
**Competitive reference**: GitHub PR file review checkboxes, Deff per-file review toggles.

---

### P2 — Nice to Have

#### Mouse Support for Navigation
**Status**: Spec written (008), agent expired — needs re-implementation
**Spec**: 008

#### Watch Mode with Auto-Refresh
**Rationale**: Competitive feature from critique and Duff. Auto-detect file changes and refresh the diff.

#### Syntax-Aware Folding
**Rationale**: Collapse unchanged functions/blocks to focus on actual changes. Would benefit from tree-sitter.

#### Diff Snapshot / Time Travel
**Rationale**: Compare agent iterations by saving diff snapshots.

#### Smart Review Ordering
**Rationale**: Order files by likely importance using heuristics (test files last, config first, etc.).

#### Glob File Filtering
**Rationale**: Filter navigator by glob patterns (e.g., `*.rs`, `src/**`). Validated by Codex CLI's path-scoped review filters feature.

#### Diff Heatmap Overlay
**Rationale**: Color-code regions by change density for quick visual scanning.

#### Inline Diff Minimap
**Status**: NEW (2026-03-17)
**Rationale**: Vertical minimap on the right edge of the diff view showing change density across the file. Color-coded bar indicating add/delete regions for quick visual orientation. Inspired by VS Code's minimap and patchcast's visual diff approach. Helps reviewers understand where changes are concentrated without scrolling.
**Scope**: Render a narrow column (2-3 chars wide) on the right edge of the diff view. Map each row to a proportional section of the file. Color green for additions, red for deletions, dim for context. Highlight current viewport position.

#### Annotation Export to GitHub PR Comments
**Status**: Promoted from P3 (2026-03-17)
**Rationale**: One-key export that posts annotations as GitHub PR review comments. Maps mdiff categories/severities to GitHub review format. Closes the loop: review in mdiff -> feedback on GitHub. Research confirms PR comments are the standard AI agent input format (GitHub Copilot coding agent consumes PR review comments to iterate). This is the missing link in mdiff's feedback pipeline.
**Scope**: GitHub API integration, map annotation categories to review comment format, batch submission as a single review, support for line-specific comments.

---

### P3 — Future / Exploratory

#### Command Transparency Log
**Rationale**: Show underlying git commands being executed (lazygit's most-loved feature).

#### External Editor Integration
**Rationale**: Open current file at current line in $EDITOR.

#### Annotation Templates
**Rationale**: Pre-defined annotation templates for common feedback patterns.

#### Diff Bookmarks (Vim-Style Marks)
**Rationale**: Bookmark specific lines/hunks with `m` + letter, jump with `'` + letter. Power user feature for navigating large changesets. Inspired by Harpoon's quick-navigation UX pattern.

#### Multi-Agent Session Comparison
**Rationale**: Inspired by 1Code's multi-agent management. Compare outputs from different agents working on the same task side-by-side. Requires session snapshot infrastructure.

#### Review Session Timer / Metrics
**Status**: NEW (2026-03-17)
**Rationale**: Track time spent per file and total review session duration. Show in feedback summary: "Spent 3min on app.rs, 45s on Cargo.toml." Useful for understanding review effort and efficiency. Could feed into structured feedback export for agent training data.

#### Evidence-Based Review Annotations
**Status**: NEW (2026-03-17)
**Rationale**: Extend annotation system to optionally include evidence/reasoning. Structured annotation includes: finding, evidence, confidence, severity. Maps directly to the agentic code review "submit review" action pattern (Archbot architecture). Would make mdiff's feedback output more useful as training signal for coding agents.

---

## Open Issues Triage

| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open | Spec 020 written, Cursor agent blocked (model unavailable) |
| #52 | Feature | P1 | Open | Spec 021 written, Cursor agent blocked (model unavailable) |
| #37 | Feature | P1 | Open (PR #45 merged, issue not closed) | Implementation complete, issue should be closed |
| #35 | Bug | P0 | Open (PR #51 merged) | Fix merged, issue should be closed |
| #25 | Bug | P0 | Open (PR #50 merged) | Fix merged, issue should be closed |
| #38 | UX/Safety | P0 | Closed | Resolved |

---

## Competitive Landscape

### Tools Tracked
- **critique** (TypeScript/Bun): TUI diff viewer with watch mode, glob filtering, word-level diff, tree-sitter syntax highlighting. v0.1.127 released 2026-03-15. 1,087 stars, 37 releases, rapid iteration. Key competitive features to track.
- **tuicr** (Rust): Interactive code review TUI, very early stage. Uses Claude AI integration.
- **acre** (Rust): Another TUI diff viewer, minimal features.
- **deff** (Rust): New Rust TUI diff viewer (HN launch 2026-03-05, 37 points). Side-by-side, vim motions, per-file review toggles. No annotation or agent feedback.
- **difi** (Go): TUI diff viewer with Neovim integration.
- **git-review** (Go): CLI code review tool.
- **justshowmediff** (Go): Zero-dependency HTML diff viewer for Claude Code/Codex headless agent workflows.
- **IPE**: Intercepts Claude Code's ExitPlanMode hook for GitHub-style code review with inline comments and approve/request-changes workflow.
- **diffreview** (Zsh): Pipes Git diff into Claude Code or GitHub Copilot CLI for AI-powered review.
- **Fresh** (Rust): New terminal editor with command palette (`Ctrl+P`), discoverable UX, extreme performance for large files. Architectural inspiration for command palette feature.
- **claudes-ai-buddies** (CLI): Multi-AI code review via confidence bidding between Claude, Codex, and Gemini.
- **1Code** (YC W26): Multi-agent management tool addressing "terminal hell" when running multiple AI coding agents. Uses git worktree isolation and GUI layer. Validates the exact pain point mdiff targets from a different angle.
- **ClaudeTUI** (v0.3): TUI statusline/monitor for Claude Code sessions. Shows growing demand for terminal-native AI agent tooling.
- **OpenCode**: Terminal-first AI coding agent (TUI + CLI). Representative of agents whose output mdiff reviews.
- **Codex CLI**: OpenAI's terminal-based coding agent. Adding path-scoped review filters and review queuing — features that validate mdiff's direction.
- **Duff** (Browser): Browser-based multi-repo Git diff viewer built for reviewing AI coding agent output. Auto-refresh, visual image diffs, commit history graphs, multi-repo workspace persistence. Different niche (browser vs TUI) but validates the same use case.
- **patchcast** (Rust): Converts git diffs into animated MP4 video walkthroughs using syntect. Novel diff visualization with scene-based animations (red-pulse deletions, green-slide additions). Different medium but interesting UX patterns.
- **Kaleidoscope**: macOS-native diff/merge tool. Excellent visual polish but no TUI, no agent integration.
- **lazygit**: Gold standard for TUI git UX. Hunk-level operations, keyboard-driven workflow. Part of "Terminal Power Trio" (Ghostty + Yazi + Lazygit).
- **fzf**: Gold standard for fuzzy search UX in terminals.
- **jjui**: TUI for jujutsu (jj), panel-based keyboard-driven paradigm.
- **gstack** (CLI): Garry Tan's open-source Claude Code toolkit with dedicated `/review` mode for production risk assessment and code review.

### Key Market Gap
No tool combines all three: (1) TUI-native diff review, (2) structured human feedback collection, (3) direct integration with coding agents. mdiff is uniquely positioned here.

### HN Signal (2026-03-05)
A commenter on the Deff HN thread explicitly requested a TUI diff tool with the ability to "comment on lines/ranges in a diff to provide targeted feedback to coding agents."

---

## Research Notes

### UX Patterns Worth Adopting
- **Contextual keybindings per panel** (lazygit, jjui)
- **Glob-based filtering in search** (fzf-lua)
- **Command transparency** (lazygit): Showing underlying commands builds trust
- **Command palette** (Fresh editor, VS Code): Fuzzy-searchable action list for discoverability
- **Disable dangerous operations by default** (lazygit community feedback)
- **Scene-based sequential review** (patchcast): Structured progression through files with visual transitions
- **Maximize current pane** shortcut (Ghostty): Focus mode within multi-panel layout

### RLHF / Active Learning Insights
- DPO, GRPO, and RLVR reducing reliance on traditional reward models. Structured annotation schemas should capture multiple signal dimensions.
- Active learning principles directly apply to deciding which hunks need review.
- High false-positive rates in AI code review cause reviewers to disengage entirely.

### Agent Feedback Patterns (from 2026-03-07 research)
- **Structured inline annotation + verdict**: IPE demonstrates that explicit approve/reject decisions improve agent feedback loops.
- **Machine-readable feedback export**: justshowmediff and diffreview show the trend toward structured, tool-consumable feedback formats.
- **Review-as-a-quality-gate**: Human code review is the single most important quality gate between AI agent output and production.

### Multi-Agent Review Patterns (from 2026-03-12 research)
- **Anthropic Claude Code Review**: Dispatches parallel specialized agents per PR, each focusing on a different issue class (type mismatches, concurrency bugs, security). Verification step filters false positives to <1% rate. Uses three-tier severity: Normal (must fix), Nit (minor), Pre-existing (not from this PR). Validates mdiff's existing annotation categories approach.
- **Fresh editor (Rust TUI)**: New terminal editor with command palette (`Ctrl+P`), discoverable UX patterns, and extreme performance for large files. Demonstrates that command palettes work well in TUI context. Directly inspired the Command Palette feature.
- **Nurture-First Development**: Research framework proposing "Knowledge Crystallization Cycles" for growing agent knowledge from operational feedback. Three-Layer Cognitive Architecture organizes agent knowledge by volatility. Relevant to mdiff's mission of structured feedback loops.
- **Growing review bottleneck**: AI-assisted code output per engineer up ~200% at companies using coding agents, creating acute demand for structured review tools. mdiff's positioning as a structured feedback tool is increasingly validated.

### Cross-Context Review & Agent Ecosystem (from 2026-03-13 research)
- **Cross-Context Review (CCR)** (arXiv 2603.12123): Reviewing LLM-generated code in a fresh session (decoupled from generation context) yields significantly better error detection (F1 28.6% vs 24.6%). Validates mdiff's design as a standalone review tool separate from the agent.
- **1Code (YC W26)**: Addresses "terminal hell" of managing multiple AI coding agents with scattered git diffs. Uses git worktree isolation. Validates the multi-agent review pain point.
- **Codex CLI review queuing**: Users requesting the ability to queue `/review` during long agent tasks. Shows demand for asynchronous review workflows.
- **Microsoft Agent Framework human-in-the-loop**: Formal pause/approve/reject pattern with structured rejection feedback passed back to agents. Architectural reference for mdiff's approve/reject workflow.
- **No direct competitor found**: No tool combines Rust TUI + git diff viewing + AI agent output review specifically. mdiff's niche remains underserved in a rapidly growing market.

### AI-Native Terminal & Inline Q&A Patterns (from 2026-03-16 research)
- **Duff** (browser-based diff viewer): Launched specifically for reviewing AI coding agent output across multiple repos. Auto-refresh, image diffs, commit history graphs. Validates mdiff's target use case from a browser angle.
- **gstack** (Garry Tan): Open-source Claude Code toolkit with dedicated `/review` and `/plan-eng-review` commands. Shows growing demand for structured review workflows layered on top of coding agents.
- **QwenLM multi-model code review**: Parallel review agents across different LLMs with arbitrator producing unified reports. Points toward multi-model review as a future pattern.
- **AI-native terminal design trend**: Awal Terminal (Rust + Swift), Ghostty forks with Claude Code integration, side panels showing AI session structure. The terminal is becoming an AI-aware workspace.
- **GitHub Copilot feedback loop**: PR-based RLHF — agent generates code as PR, humans review and leave comments, agent iterates. The pull request is the structured annotation interface. mdiff's annotation system maps directly to this pattern.
- **CursorBench-3**: Evaluation suite measuring coding agent quality on real developer interactions. Shows growing investment in measuring agent output quality — the exact problem mdiff's feedback system addresses.

### Structured Feedback & Agent Instruction Patterns (from 2026-03-17 research)
- **Agentic review architecture (Archbot)**: Shift from fixed LLM pipelines to agentic loops with structured "submit review" actions. Evidence-fetching loops address context gaps that cause confidently wrong reviews. Validates mdiff's structured annotation approach.
- **PR comments as agent input format**: GitHub Copilot coding agent consumes structured PR review comments to iterate on code. PR comment threads are now a de facto machine-readable feedback format. Directly validates mdiff's Annotation Export to GitHub PR Comments feature.
- **Agent instruction file ecosystem**: CLAUDE.md, AGENTS.md, copilot-instructions.md, GEMINI.md, CODEX.md files emerging as structured context format for AI agents. Review output could target these same formats.
- **critique v0.1.127**: Reached 1,087 stars with 37 releases. Now uses tree-sitter for syntax parsing. Key features: watch mode, glob filtering, split view, word-level diff. Active and iterating fast — competitive pressure increasing.
- **patchcast (Rust)**: Novel approach converting git diffs to animated MP4 walkthroughs. Scene-based progression (title card, code reveal, deletion highlight, addition highlight) is an interesting UX pattern for sequential review.
- **Terminal Power Trio pattern**: Ghostty + Yazi + Lazygit emerging as the standard power-user terminal workflow. mdiff should integrate well with this ecosystem.

---

## Changelog

### 2026-03-17
- **PROMOTED Review Progress Bar to P1**: Moved from P2 — high impact for preventing missed files in large changesets, reasonable implementation scope
- **PROMOTED Annotation Export to GitHub PR Comments to P2**: Moved from P3 — research confirms PR comments are the standard AI agent input format, closes the feedback loop
- **NEW P2**: Inline Diff Minimap — vertical change density indicator on right edge of diff view
- **NEW P3**: Review Session Timer / Metrics — track time spent per file for feedback metadata
- **NEW P3**: Evidence-Based Review Annotations — structured finding/evidence/confidence annotations
- **Cursor agents blocked again**: All 3 agents (File Tree Navigator, Inline Q&A, Diff Statistics Dashboard) failed to create — model unavailable for 2nd consecutive run
- **Updated P1 statuses**: Added model unavailability dates to #53, #52, and Diff Statistics Dashboard
- **Research**: critique v0.1.127 (1,087 stars, tree-sitter), patchcast (Rust diff-to-video), agentic review architecture (Archbot), PR comments as agent input format, agent instruction file ecosystem, Terminal Power Trio pattern
- Added patchcast to competitive landscape
- Updated critique entry with latest stats
- Added "Structured Feedback & Agent Instruction Patterns" research section
- PR #54 (release v0.1.19) is the only open PR — release automation, no action needed

### 2026-03-16
- **PROMOTED Issue #53 to P1**: File Tree Navigator — spec 020 written with detailed tree data structures, box-drawing indent guides, directory collapse/expand, zM/zR fold commands
- **PROMOTED Issue #52 to P1**: Ask a Question (Inline Q&A) — spec 021 written with context assembly, answer panel, Q&A history, session persistence
- **Cursor agents queued** for 3 features: #53 (File Tree Navigator), #52 (Inline Q&A), Diff Statistics Dashboard — model temporarily unavailable
- **Updated P0 statuses**: #25 FIXED (PR #50 merged), #35 FIXED (PR #51 merged), #38 CLOSED
- **Updated Open Issues Triage**: 2 new issues (#52, #53) from repo owner, 3 existing issues (#25, #35, #37) have fixes merged and should be closed
- **Research**: Duff browser-based diff viewer (direct competitor validation), gstack code review CLI, QwenLM multi-model review, AI-native terminal design trend, GitHub Copilot feedback loop patterns, CursorBench-3
- Added Duff, gstack to competitive landscape
- Added "AI-Native Terminal & Inline Q&A Patterns" research section
- No open PRs to review (all merged since last run)

### 2026-03-13
- **Cursor agents launched** for all 3 open P0 issues: #25 (diff line calc), #35 (global search UX), #38 (remove q keybinding)
- **ADDED P1 #6 spec**: Diff Statistics Dashboard — spec 019 written, provides changeset overview with file-level stats, sparkline bars, sort modes, and jump-to-file
- **ADDED P3**: Multi-Agent Session Comparison — inspired by 1Code's multi-agent management
- **Updated Open Issues Triage**: Issue #37 still open despite PR #45 being merged — needs manual closure
- **Research**: Cross-Context Review paper (decoupled review improves error detection), 1Code multi-agent management, Codex CLI review queuing, Microsoft Agent Framework human-in-the-loop patterns
- Added 1Code, ClaudeTUI, OpenCode, Codex CLI to competitive landscape
- Updated "Glob File Filtering" rationale with Codex CLI path-scoped review reference
- Added "Cross-Context Review & Agent Ecosystem" research section

### 2026-03-12
- **ADDED P0 #5**: Remove `q` keybinding (Issue #38) — spec 015, Cursor agent pending
- **ADDED P1**: Command Palette (`Ctrl+P`) for action discoverability — spec 018 written, Cursor agent pending
- **Updated Open Issues Triage**: 4 open issues reviewed (#25, #35, #37, #38)
  - #37 (Opencode models): RESOLVED — PR #45 merged
  - #35 (Global search bug): Spec 013 exists, Cursor agent queued
  - #38 (Remove q keybinding): Spec 015 exists, Cursor agent queued
  - #25 (Diff line calculations): Spec 009 exists, still open
- **Reviewed recent merges**: PRs #41-#47 all merged since last run (v0.1.16 released)
- **Research**: Anthropic multi-agent code review (3-tier severity), Fresh Rust editor (command palette UX), Nurture-First Development framework
- **Cursor agent creation blocked**: Model unavailable — 3 agents queued for next run
- Added Fresh editor, claudes-ai-buddies to competitive landscape
- Added "Multi-Agent Review Patterns" research section

### 2026-03-07
- **PROMOTED Issue #25 to P0** (#4): Diff line calculations bug — spec 009, Cursor agent launched
- **ADDED P1 #14**: Word-Level Diff Highlighting — spec 010, Cursor agent launched
- **ADDED P1 #15**: Approve/Reject Agent Workflow — spec 011, Cursor agent launched
- **ADDED P1 #16**: Fix cmd+K Kill Wrong Session (Issue #24) — needs investigation
- Added justshowmediff, IPE, diffreview to competitive landscape
- Added "Agent Feedback Patterns" section to research notes

### 2026-03-06
- Added P1 items #11 (Review Checklist Templates), #12 (Diff Complexity Indicators)
- Added P2 items (Mouse Support, Watch Mode, Syntax-Aware Folding, Diff Snapshot)
- Updated statuses: #1 Hunk Navigation -> Merged, #10 Which-Key -> Merged
- Added Deff to competitive landscape
- Created specs 006, 007, 008

### 2026-03-05
- Added P1 items #8 (Quick-Reactions), #9 (Feedback Summary View), #10 (Which-Key)
- Added P2 items (Smart Review Ordering, Glob File Filtering, Diff Heatmap)
- Added P3 items (Command Transparency Log, External Editor Integration)
- Created specs 003, 004, 005

### 2026-03-04
- Initial roadmap created
- Opened PRs for P0 items #1, #2, #3
- Competitive analysis of critique, tuicr, acre, Kaleidoscope, lazygit, fzf
- Researched RLHF/human feedback patterns for annotation design
