# mdiff Roadmap & Feature Backlog

> **Maintained by**: Milo (automated ideation agent)
> **Last updated**: 2026-03-08
> **Schedule**: New ideas are evaluated and prioritized every 24 hours

This document tracks feature ideas, prioritized issues, and the rationale behind them. It draws from competitive analysis of tools like **critique**, **tuicr**, **acre**, **difi**, **deff**, **git-review**, **Kaleidoscope**, **lazygit**, **fzf**, **justshowmediff**, **IPE**, **diffreview**, **git-lanes**, **claude-compaction-viewer**, and patterns from RLHF/human feedback research.

---

## Priority Tiers

### P0 — Critical Path (Active PRs / Bugs)

#### 1. Hunk-Level Navigation (`]` / `[` keys)
**Status**: Merged
**Rationale**: Every serious diff tool (vim-fugitive, lazygit, tig, delta) supports jumping between hunks. Currently mdiff only has line-by-line (`j`/`k`) and page scrolling. For large agent-generated diffs with hundreds of lines, reviewers need to skip context and jump between actual changes.
**Scope**: Add `JumpNextHunk` / `JumpPrevHunk` actions, remap existing `]`/`[` annotation navigation to `Ctrl+]`/`Ctrl+[`, display "Hunk N/M" in the HUD, support cross-file hunk jumping.
**Competitive reference**: vim `[c`/`]c`, lazygit hunk navigation, tuicr `{`/`}` keys.

#### 2. Annotation Categories & Severity Levels
**Status**: PR #12 open (draft, targets main — STALE, needs retarget to develop or close)
**Rationale**: Structured feedback is dramatically more effective than free-text when communicating with coding agents. Research on RLHF shows categorized, severity-tagged annotations lead to better model responses. This transforms mdiff from a "leave comments" tool into a structured feedback system.
**Scope**: Add category picker (Bug/Style/Performance/Security/Suggestion/Question/Nitpick) and severity levels (Critical/Major/Minor/Info) to the annotation flow. Single-keypress selection. Update prompt template to include structured metadata. Backward-compatible with existing sessions.
**Known issue**: Session persistence hardcodes category=Suggestion, severity=Minor for loaded annotations. Needs AnnotationEntry format update.
**Competitive reference**: GitHub PR review comment types, V7 Labs RLHF annotation categories, Linear issue priority levels.

#### 3. Global Fuzzy Search Across All Diff Content (`Ctrl+F`)
**Status**: PR #13 closed
**Rationale**: The existing `/` search only filters file names in the navigator. When reviewing large agent changesets spanning 20+ files, you need to find specific code patterns, variable names, or string literals across ALL changed files. The `nucleo` crate is already in Cargo.toml but only used for file name filtering.
**Scope**: Add `Ctrl+F` for global content search across all diff hunks, using nucleo for fuzzy matching with an exact-match toggle (`Ctrl+G`). Live incremental results, match highlighting in diff view, "Match N/M" counter, cross-file navigation with `n`/`N`.
**Competitive reference**: fzf search patterns, VS Code Ctrl+Shift+F, Kaleidoscope full-text search.

#### 4. Fix Diff Line Calculations (Issue #25)
**Status**: MERGED (PR #26)
**Addresses**: GitHub Issue #25
**Rationale**: When using `G` (ScrollToBottom) in the diff view, the viewport did not render the full diff. Lines below the viewport boundary were inaccessible.
**Resolution**: Fixed scroll bound calculation to match visual row count.

#### 5. Fix Which-Key Dialog Flashing (Issue #27) — NEW
**Status**: Spec written (010-which-key-flash-fix), Cursor agent launched
**Addresses**: GitHub Issue #27
**Rationale**: Pressing `?` opens the which-key dialog, but it flashes and disappears immediately. Crossterm emits both KeyPress and KeyRelease events; only Press should be handled. This completely breaks the discoverability feature.
**Scope**: Filter `KeyEventKind::Press` only in EventReader. One-line fix with major UX impact.
**Root cause**: EventReader forwards KeyRelease events, causing toggle to fire twice.

---

### P1 — High Impact

#### 6. File Tree Navigator with Directory Grouping
**Rationale**: When agents modify 30+ files across multiple directories, the flat file list becomes unwieldy. A collapsible tree view grouped by directory (like VS Code's explorer) would make navigation much faster.
**Scope**: Add tree view mode to the navigator (toggle with `T`), collapse/expand directories, show file counts per directory, sort by path or by change type.
**Competitive reference**: VS Code file explorer, GitHub PR file tree, lazygit file tree.

#### 7. Diff Statistics Dashboard
**Rationale**: Before diving into line-by-line review, reviewers need an overview: how many files changed, total additions/deletions, which files have the most churn. This is especially important for agent output where you want to quickly assess if the agent went off track.
**Scope**: Add a summary view (toggle with `S`) showing: total files/additions/deletions, per-file sparkline bars, file type breakdown, largest files by change size, annotation coverage stats.
**Competitive reference**: GitHub PR stats bar, `git diff --stat`, diffray summary.

#### 8. Configurable Keybinding System
**Rationale**: Power users expect to customize keybindings. As mdiff adds more features, key conflicts become inevitable. A TOML-based keybinding system would let users remap any action.
**Scope**: Add `[keybindings]` section to config.toml. Map action names to key combos. Support modifier keys (Ctrl, Shift, Alt). Default config ships with current mappings.
**Competitive reference**: lazygit custom keybindings, neovim keymap, VS Code keybindings.json.

#### 9. Review Checklist Templates
**Status**: PR #21 (Cursor agent finished, PR targets main — needs re-implementation targeting develop)
**Rationale**: Checklists dramatically improve annotation consistency and reduce "forgetting to check" failure mode. RLHF research confirms structured review protocols lead to better outcomes.
**Scope**: Configurable per-project review checklists loaded from config.toml, toggleable panel UI, session persistence, prompt output integration.
**Spec**: 006-review-checklist.md

#### 10. Diff Complexity Indicators
**Status**: MERGED (PR #22 to develop)
**Rationale**: Heuristic-based complexity scoring helps reviewers triage large changesets by identifying high-risk hunks.
**Spec**: 007-complexity-indicators.md

#### 11. Structured Feedback Export (`Ctrl+E`)
**Status**: Spec written (011-structured-feedback-export), Cursor agent launched
**Rationale**: The agentic AI ecosystem needs machine-parseable feedback. mdiff collects rich structured data (annotations, scores, review status) but has no export path. JSON export enables CI pipelines, agent training loops, and RLHF data collection.
**Scope**: `Ctrl+E` exports full review session as JSON to `.mdiff-feedback/` directory. Schema includes annotations with categories/severity, line scores, review status, and review completeness metrics.
**Competitive reference**: review-for-agent (Waraq-Labs) JSON/Markdown export, REEL pattern "state on disk".
**Spec**: 011-structured-feedback-export.md

#### 12. Approve/Reject Agent Workflow
**Status**: Spec written (011-approve-reject-workflow)
**Rationale**: After reviewing, there's no explicit decision point. Research on IPE and GitHub's PR review model shows explicit approve/reject verdicts dramatically improve feedback loops. Forces deliberate judgment, structures exported feedback with a verdict, enables tracking approval rates.
**Scope**: `A` opens review decision dialog with Approve/Request Changes/Reject options, summary comment, auto-export to clipboard, session persistence, HUD badge.
**Spec**: 011-approve-reject-workflow.md

#### 13. Word-Level Diff Highlighting
**Status**: MERGED (PR #28 to develop)
**Rationale**: Character-level diff highlighting within lines is table-stakes UX for 2026 diff viewers.
**Spec**: 010-word-level-diff.md

#### 14. Fix cmd+K Killing Wrong Agent Session (Issue #24)
**Status**: Spec written (012-agent-session-kill-fix), Cursor agent launched
**Addresses**: GitHub Issue #24
**Rationale**: `Ctrl+K` in the Agent Outputs tab kills the wrong session due to index mapping mismatch between display order and backing vector.
**Scope**: Audit and fix index resolution in KillAgentProcess handler. Ensure kill targets the displayed selection.
**Spec**: 012-agent-session-kill-fix.md

---

### P2 — Quality of Life

#### 15. Mouse Support for Navigation
**Status**: MERGED (PR #30 to main)
**Rationale**: Scroll wheel and click-to-select reduce friction for users who aren't pure keyboard navigators.

#### 16. Smart Review Ordering
**Rationale**: Automatically order files by review priority based on complexity, change size, and file type. Present the most impactful files first.
**Competitive reference**: RLHF active learning principles (select most informative data points for human labeling).

#### 17. Glob File Filtering
**Rationale**: Let users filter the file list by glob patterns (e.g., `*.rs`, `src/**`). Useful for focusing on specific parts of large changesets.
**Competitive reference**: critique's glob filtering feature.

#### 18. Diff Heatmap Overlay
**Rationale**: Color-code the file navigator by change density. Files with more changes get warmer colors. Provides instant visual triage signal.

#### 19. Watch Mode with Auto-Refresh
**Rationale**: When working with agents that iterate on code, auto-refresh the diff when files change on disk. critique already ships this.
**Competitive reference**: critique watch mode, entr/watchexec patterns.

#### 20. Syntax-Aware Folding
**Rationale**: Fold unchanged functions/blocks to show only the changed code in context. Requires tree-sitter integration (already in Cargo.toml for highlighting).
**Competitive reference**: difftastic structural diffing, VS Code fold/unfold.

#### 21. Diff Snapshot / Time Travel
**Rationale**: Save snapshots of review sessions at different points in time. Compare agent iterations side by side.

#### 22. Bookmark/Pin Lines for Quick Return — NEW
**Rationale**: During review of large diffs, reviewers frequently lose their position when switching between files or scrolling. Vim-style marks (`ma`, `'a`) let users bookmark specific lines and jump back instantly. Essential for power-user review workflows with 50+ file changesets.
**Scope**: Press `B` to bookmark current cursor position (file + line). Press `'` to cycle through bookmarks. Show bookmark indicators in the gutter. Persist bookmarks in session file. Support up to 26 named bookmarks (a-z).
**Competitive reference**: vim marks, VS Code bookmarks extension, IntelliJ bookmarks.

#### 23. Review Progress Bar / Session Timer — NEW
**Rationale**: Reviewers have no awareness of review pacing or completeness. A progress indicator showing "12/30 files seen, 8 annotated" plus elapsed time helps reviewers pace themselves and know when they're done. RLHF annotation throughput research shows that awareness of progress improves annotation quality.
**Scope**: Add a persistent progress bar to the HUD showing files viewed/total, annotation count, and elapsed time. Track time-per-file for post-review analytics. Include in structured feedback export.

#### 24. Annotation Quick-Reply Templates — NEW
**Rationale**: Reviewers frequently leave the same types of comments: "Handle error properly", "Add tests for this", "Consider edge case X". Pre-configured templates reduce typing for common feedback patterns, similar to GitHub's saved replies.
**Scope**: Press `T` + number (1-9) to insert a template annotation. Templates loaded from config.toml. Templates include default category/severity. Customizable per-project.
**Competitive reference**: GitHub saved replies, Linear quick actions.

#### 25. Export to GitHub PR Comment — NEW
**Rationale**: After reviewing in mdiff, feedback needs to reach the agent or team. Currently the only output is clipboard copy. Direct export to a GitHub PR comment (using the GitHub API) would close the loop between TUI review and the PR-based workflow that most teams use.
**Scope**: Press `Ctrl+G` to export review as a GitHub PR comment. Requires GitHub token in config. Format as structured Markdown comment with annotations, scores, and verdict.
**Competitive reference**: review-for-agent structured export, gh CLI PR comment workflow.

---

### P3 — Future / Exploratory

#### 26. Side-by-Side Agent Comparison View — NEW
**Rationale**: RLHF research shows humans compare outputs more easily than they evaluate individual outputs. When evaluating multiple coding agents (e.g., Claude vs GPT vs Gemini on the same task), a diff-of-diffs view showing what each agent changed would enable direct comparison. This is a novel capability no current tool offers.
**Scope**: Load two agent branches/commits, compute diff-of-diffs, render side-by-side with per-agent color coding. Major architectural effort.
**Competitive reference**: RLHF preference comparison interfaces, A/B testing tools.

#### 27. Command Transparency Log
**Rationale**: Show a log of all actions taken during the review session. Useful for debugging keybinding issues and understanding review workflows.

#### 28. External Editor Integration
**Rationale**: Open the current file at the current line in $EDITOR (nvim, helix, etc.) for quick edits during review.

---

## Competitive Landscape

| Tool | Language | TUI? | Agent Feedback? | Key Differentiator |
|------|----------|------|----------------|---------------------|
| **mdiff** | Rust | Yes | Yes | Structured annotations, scores, checklists, export |
| **deff** | Rust | Yes | No | Side-by-side, vim motions, per-file review toggle |
| **critique** | TypeScript | Yes | No | Watch mode, word-level diff, glob filtering |
| **lazygit** | Go | Yes | No | Full git UI, 70k+ stars, mature UX patterns |
| **delta** | Rust | Pager | No | Syntax highlighting, side-by-side, git integration |
| **difftastic** | Rust | Pager | No | Structural/syntax-aware diffing |
| **justshowmediff** | Go | Browser | No | Zero-dep HTML output, Claude Code hooks |
| **diffreview** | Zsh | No | Yes (LLM) | Pipes diffs to Claude/Copilot for AI review |
| **git-lanes** | TypeScript | No | No | Parallel agent isolation, worktree management |
| **review-for-agent** | TypeScript | Browser | Yes | GitHub-style UI, structured JSON/Markdown export |
| **difit** | TypeScript | Browser | No | Local web server GitHub-like diff view |
| **claude-compaction-viewer** | Python | Yes | No | Claude Code session inspection |

### Key Competitive Signals (2026-03-08)
- **Deff on HN** (47169518): 37+ points, 19 comments. Commenters explicitly requesting inline annotation for agent feedback — validates mdiff's core value proposition. Deff has no annotation features.
- **review-for-agent**: Structured JSON/Markdown export pattern validates our spec 011 (structured feedback export). Their approach: local web server with GitHub-style diff view.
- **git-lanes**: Multi-agent workflow isolation tool (12 stars), showing the ecosystem is expanding toward parallel agent management.
- **claude-compaction-viewer**: TUI for inspecting Claude Code sessions — growing niche of AI-agent-specific developer tools.
- **justshowmediff**: Browser-based alternative approach, targeting Claude Code/Codex post-tool hooks.

---

## Research Notes

### RLHF / Active Learning Insights
- DPO, GRPO, and RLVR paradigms reducing reliance on traditional reward models, but structured human preferences remain the gold standard for nuanced feedback.
- Active learning principle: select the most informative data points for human review. Complexity indicators (#10) implement a version of this for diff review.
- **Key insight**: Humans compare outputs more easily than they evaluate individual outputs. Ranking-based interfaces (A/B comparison) are more effective than absolute quality scoring. This informs the Agent Comparison View concept (#26).
- RLHF code evaluation protocols use predefined issue categories (instruction adherence, code logic, problem-solving quality) — directly maps to annotation categories in mdiff.

### AI Code Review Failure Modes (2026-03-08)
- High false-positive rates in AI code review cause reviewers to disengage entirely. Complexity indicators help avoid the "everything looks important" failure mode.
- AI-generated code requires fundamentally different review workflows: no implicit context from author, high confidence without correctness, volume overwhelming attention. mdiff addresses all three.
- The REEL (Review Evaluated Engineering Loop) pattern formalizes: spec-in, fresh sessions per task, acceptance gates. mdiff's verdict system (spec 011) maps directly to the acceptance gate step.

### Agent Feedback Patterns (from 2026-03-08 research)
- **Structured inline annotation + verdict**: IPE demonstrates that explicit approve/reject decisions improve agent feedback loops. mdiff now implements this with the Approve/Reject workflow (#12).
- **Machine-readable feedback export**: justshowmediff and diffreview show the trend toward structured, tool-consumable feedback formats. Structured export (spec 011) addresses this directly.
- **Review-as-a-quality-gate**: The 2026 consensus is that human code review is the single most important quality gate between AI agent output and production. Scoring, checklists, and verdicts all formalize this gate.
- **Layered review architecture**: Automated first-pass (lint, types, naming) + human second-pass (architecture, design, business logic). mdiff operates at the human layer.

---

## Open Issues Triage (2026-03-08)

| Issue | Type | Priority | Status | Action |
|-------|------|----------|--------|--------|
| #27 | Bug | P0 | Spec ready | Cursor agent launched this cycle |
| #25 | Bug | P0 | RESOLVED | PR #26 merged |
| #24 | Bug | P1 | Spec ready | Cursor agent launched this cycle |

---

## Changelog

### 2026-03-08
- **PROMOTED Issue #27 to P0** (#5): Which-key dialog flash bug — spec 010, Cursor agent launched
- **ADDED P1 #14**: Fix cmd+K Kill Wrong Session (Issue #24) — spec 012, Cursor agent launched
- **LAUNCHED** Cursor agent for Structured Feedback Export (spec 011)
- Updated Issue #25 status to RESOLVED (PR #26 merged)
- Updated statuses: #10 Complexity Indicators -> Merged, #13 Word-Level Diff -> Merged, #15 Mouse Support -> Merged
- Noted PRs #12 and #14 are STALE drafts targeting main
- **NEW P2 ideas**: Bookmark/Pin Lines (#22), Review Progress Bar (#23), Annotation Quick-Reply Templates (#24), Export to GitHub PR Comment (#25)
- **NEW P3 idea**: Side-by-Side Agent Comparison View (#26)
- Added git-lanes, claude-compaction-viewer, review-for-agent, difit to competitive landscape
- Updated research notes with RLHF comparative interface insights, REEL pattern, layered review architecture
- Added "AI Code Review Failure Modes" research section

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
