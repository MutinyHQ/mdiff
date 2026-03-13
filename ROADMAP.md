# mdiff Roadmap & Feature Backlog

> **Maintained by**: Milo (automated ideation agent)
> **Last updated**: 2026-03-13
> **Schedule**: New ideas are evaluated and prioritized every 24 hours

This document tracks feature ideas, prioritized issues, and the rationale behind them. It draws from competitive analysis of tools like **critique**, **tuicr**, **acre**, **difi**, **deff**, **git-review**, **Kaleidoscope**, **lazygit**, **fzf**, **justshowmediff**, **IPE**, **diffreview**, **Fresh**, **1Code**, and patterns from RLHF/human feedback research.

---

## Priority Tiers

### P0 — Critical Path (Active PRs / Bugs)

#### 1. Hunk-Level Navigation (`]` / `[` keys)
**Status**: Merged

#### 2. Annotation Categories & Severity Levels
**Status**: Merged

#### 3. Global Fuzzy Search Across All Diff Content (`Ctrl+F`)
**Status**: Merged (core feature). **UX Bug open** — Issue #35, Spec 013, Cursor agent launched (2026-03-13).
**Known issue**: `n` key conflict prevents typing 'n' in search query. Navigation being moved to Ctrl+N/Ctrl+P.

#### 4. Fix Diff Line Calculations (Issue #25)
**Status**: Spec written (009), Cursor agent launched (2026-03-13)
**Addresses**: GitHub Issue #25
**Rationale**: When using `G` (ScrollToBottom) in the diff view, the viewport does not render the full diff. Lines below the viewport boundary are inaccessible. Critical usability bug.

#### 5. Remove `q` Keybinding for Quit (Issue #38)
**Status**: Spec written (015), Cursor agent launched (2026-03-13)
**Addresses**: GitHub Issue #38
**Rationale**: Pressing `q` immediately quits mdiff without any confirmation, even when there are staged annotations. This is a data loss risk. The `q` key is adjacent to common review actions (`w`, `a`, `s`). Filed by repo owner — explicit request to use only Ctrl+C/Ctrl+D for exit.
**Scope**: Remove `q` -> `Quit` mapping from event.rs global bindings, update which-key overlay.

---

### P1 — High Impact

#### 5. File Tree Navigator with Directory Grouping
**Rationale**: When agents modify 30+ files across multiple directories, the flat file list becomes unwieldy. A collapsible tree view grouped by directory would make navigation much faster.
**Scope**: Add tree view mode to the navigator (toggle with `T`), collapse/expand directories, show file counts per directory.
**Competitive reference**: VS Code file explorer, GitHub PR file tree, lazygit file tree.

#### 6. Diff Statistics Dashboard
**Status**: Spec written (019), Cursor agent pending — NEW (2026-03-13)
**Rationale**: Before diving into line-by-line review, reviewers need an overview: how many files changed, total additions/deletions, which files have the most churn. No way to quickly assess changeset scope without scrolling through every file.
**Scope**: Add a summary view (toggle with `S`) showing: total files/additions/deletions, per-file sparkline bars, file type breakdown, largest files by change size. Jump-to-file from the dashboard.
**Competitive reference**: GitHub PR stats bar, `git diff --stat`, diffray summary.

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

---

### P2 — Nice to Have

#### Mouse Support for Navigation
**Status**: Spec written (008), agent expired — needs re-implementation
**Spec**: 008

#### Watch Mode with Auto-Refresh
**Rationale**: Competitive feature from critique. Auto-detect file changes and refresh the diff.

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

#### Review Progress Bar
**Rationale**: Show per-file and overall review completion percentage. Track files viewed, annotated, and marked reviewed. Visual progress bar in navigator header. Motivates completion and prevents missed files.

---

### P3 — Future / Exploratory

#### Command Transparency Log
**Rationale**: Show underlying git commands being executed (lazygit's most-loved feature).

#### External Editor Integration
**Rationale**: Open current file at current line in $EDITOR.

#### Annotation Templates
**Rationale**: Pre-defined annotation templates for common feedback patterns.

#### Annotation Export to GitHub PR Comments
**Rationale**: One-key export that posts annotations as GitHub PR review comments. Maps mdiff categories/severities to GitHub review format. Closes the loop: review in mdiff -> feedback on GitHub.

#### Diff Bookmarks (Vim-Style Marks)
**Rationale**: Bookmark specific lines/hunks with `m` + letter, jump with `'` + letter. Power user feature for navigating large changesets.

#### Multi-Agent Session Comparison
**Rationale**: Inspired by 1Code's multi-agent management. Compare outputs from different agents working on the same task side-by-side. Requires session snapshot infrastructure.

---

## Open Issues Triage

| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #25 | Bug | P0 | Open | Spec 009 written, Cursor agent launched 2026-03-13 |
| #35 | Bug | P0 | Open | Spec 013 written, Cursor agent launched 2026-03-13 |
| #37 | Feature | P1 | Open (PR #45 merged, issue not closed) | Implementation complete, issue should be closed |
| #38 | UX/Safety | P0 | Open | Spec 015 written, Cursor agent launched 2026-03-13 |

---

## Competitive Landscape

### Tools Tracked
- **critique** (TypeScript/Bun): TUI diff viewer with watch mode, glob filtering. Competitive features to track.
- **tuicr** (Rust): Interactive code review TUI, very early stage. Uses Claude AI integration.
- **acre** (Rust): Another TUI diff viewer, minimal features.
- **deff** (Rust): New Rust TUI diff viewer (HN launch 2026-03-05, 37 points). Side-by-side, vim motions, per-file review toggles. No annotation or agent feedback.
- **difi** (Rust): Inline diff viewer, simple design.
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
- **Kaleidoscope**: macOS-native diff/merge tool. Excellent visual polish but no TUI, no agent integration.
- **lazygit**: Gold standard for TUI git UX. Hunk-level operations, keyboard-driven workflow.
- **fzf**: Gold standard for fuzzy search UX in terminals.
- **jjui**: TUI for jujutsu (jj), panel-based keyboard-driven paradigm.

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

---

## Changelog

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
