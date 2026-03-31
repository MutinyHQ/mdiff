# mdiff Roadmap & Feature Backlog

> **Maintained by**: Milo (automated ideation agent)
> **Last updated**: 2026-03-30
> **Schedule**: New ideas are evaluated and prioritized every 24 hours

This document tracks feature ideas, prioritized issues, and the rationale behind them. It draws from competitive analysis of tools like **critique**, **tuicr**, **acre**, **difi**, **deff**, **git-review**, **Kaleidoscope**, **lazygit**, **fzf**, **justshowmediff**, **IPE**, **diffreview**, **Fresh**, **1Code**, **Duff**, **patchcast**, **Anduin**, **carn**, **ftdv**, **keifu**, **dead-ringer**, **cmux**, **Muse/Manyana**, **oh-my-pi**, **Superset**, **Orchestrator**, **vibetracer**, **OpenClaw**, and patterns from RLHF/human feedback research.

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

#### 6. Tree File Viewer Inconsistencies (Issue #64)
**Status**: Spec written (029), Cursor agent launched (2026-03-24)
**Addresses**: GitHub Issue #64
**Rationale**: Owner-filed bug report. The tree view (PR #56) has 5 inconsistencies: `m` keybinding broken, directory collapse needs `x` key, tree/flat preference not persistent, mouse clicks select wrong file, gitignored files shown. These regressions make the tree view unreliable for the core navigation use case.
**Scope**: Fix `m` key in tree mode, add `x` for collapse, persist tree_mode in settings, fix mouse click targeting, filter tree to only show files from diff.deltas.

#### 7. File Preview on Hover in Tree View (Issue #65)
**Status**: Spec written (030), Cursor agent launched (2026-03-24)
**Addresses**: GitHub Issue #65
**Rationale**: Owner-filed bug. Moving cursor over files in tree view doesn't update the diff pane preview — it shows the last selected file. In flat view this works correctly. This forces an extra Enter keystroke per file, doubling navigation cost when scanning large changesets.
**Scope**: Sync `diff.selected_file` with the highlighted tree entry on cursor movement. Directory entries skip the sync.

---

### P1 — High Impact

#### 5. File Tree Navigator with Directory Grouping (Issue #53)
**Status**: **Implemented** — PR #56 merged (2026-03-20). Spec 020, Cursor agent bc-f6fc2377.
**Addresses**: GitHub Issue #53
**Rationale**: When agents modify 30+ files across multiple directories, the flat file list becomes unwieldy. A collapsible tree view grouped by directory would make navigation much faster. Filed by repo owner with detailed acceptance criteria.
**Scope**: Add tree view mode to the navigator (toggle with `T`), collapse/expand directories with Enter, show file counts per directory, box-drawing indent guides, zM/zR for fold all/unfold all.
**Competitive reference**: VS Code file explorer, GitHub PR file tree, lazygit file tree, Yazi TUI file manager.

#### 6. Diff Statistics Dashboard
**Status**: Spec written (019), Cursor agent re-launched (2026-03-30).
**Rationale**: Before diving into line-by-line review, reviewers need an overview: how many files changed, total additions/deletions, which files have the most churn. No way to quickly assess changeset scope without scrolling through every file.
**Scope**: Add a summary view (toggle with `S`) showing: total files/additions/deletions, per-file sparkline bars, file type breakdown, largest files by change size. Jump-to-file from the dashboard.
**Competitive reference**: GitHub PR stats bar, `git diff --stat`, diffray summary.

#### Ask a Question: Inline Q&A Over Diff Context (Issue #52)
**Status**: Spec written (021), Cursor agent re-launched (2026-03-30).
**Addresses**: GitHub Issue #52
**Rationale**: Reviewers frequently encounter unfamiliar code or unclear intent during review and must leave the TUI to ask an LLM. This destroys flow state. Inline Q&A with diff context assembly eliminates the context switch entirely. No other TUI diff viewer offers this — a clear differentiator. Filed by repo owner.
**Scope**: `Ctrl+Q` trigger, question input bar at bottom of diff view, context assembly from visible hunks/selection, answer panel as right-side split, Q&A history with Ctrl+]/Ctrl+[, session persistence.
**Competitive reference**: Cursor IDE inline chat, Aider terminal AI assistant. No TUI diff viewer competitor has this.

#### Go-to-Line & Fuzzy File Picker (Issue #63)
**Status**: Spec written (031), Cursor agent launched (2026-03-24)
**Addresses**: GitHub Issue #63
**Rationale**: Owner-filed UX improvement. Two standard power-user navigation shortcuts missing: `:<number>` to jump to a line in the diff view (vim convention), and `Ctrl+P` for fuzzy file search modal. Both are expected by vim/VS Code users and reduce navigation friction in large changesets. Partially overlaps with Command Palette spec (018) — the file picker IS the first phase.
**Scope**: Vim-style command bar (`:` prefix, number -> go-to-line, `help` -> settings), fuzzy file picker modal with nucleo crate matching, real-time filtering, match highlights.
**Competitive reference**: VS Code Ctrl+P, Sublime Text Cmd+P, vim `:<number>`, Fresh editor command palette.

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
**Rationale**: mdiff has grown to 50+ distinct actions. The which-key overlay shows keybindings for the current context, but users must memorize bindings or scan a static list. A fuzzy-search command palette (triggered by `Ctrl+P`) would make all actions instantly discoverable and executable without memorizing keybindings. Standard UX pattern validated by VS Code, Fresh editor, Sublime Text, and every modern editor.
**Scope**: Fuzzy-search popup listing all available actions with their keybindings. Execute selected action directly. Filter by typing. Show action descriptions. Use nucleo crate for fuzzy matching (already available in the project).

#### Review Progress Bar
**Status**: Spec written (022), Cursor agent launched (2026-03-18). Promoted from P2 (2026-03-17)
**Rationale**: Prevents missed files in large changesets. Show per-file and overall review completion percentage. Track files viewed vs total, display progress bar in navigator.
**Scope**: Progress bar in navigator footer showing reviewed/total with block characters and percentage. Completion celebration state when all files reviewed.

#### Agent Attribution Labels (Hunk-Level Agent Session Markers)
**Status**: Spec written (026), Cursor agent launched (2026-03-23)
**Rationale**: When reviewing multi-agent or iterative agent output, there's no visual signal indicating which agent session produced each change. Labeling hunks with session attribution lets reviewers filter and focus on specific iterations. Inspired by Muse/Manyana's conflict-aware diff labels.
**Scope**: Gutter attribution markers with session colors, hunk header labels, session legend, filter-by-session (A key), manual tagging (Ctrl+T in visual mode). Commit-based attribution detection.

#### Agent Guidance Export (Structured Feedback as Agent Instructions)
**Status**: Spec written (027), Cursor agent launched (2026-03-23)
**Rationale**: mdiff exports annotations as JSON/markdown for humans, but agents need differently structured feedback. Agent-RLVR research shows structured guidance improves agent success 2-3x. New export formats: Agent Instruction prompt, CLAUDE.md patch, and Git trailers.
**Scope**: Three new export formats in Ctrl+E dialog. Annotation-to-guidance mapping. Format picker UI.

#### Automated Review Surface (Issue #71)
**Status**: Spec written (032), Cursor agent launched (2026-03-27)
**Addresses**: GitHub Issue #71
**Rationale**: After #70 landed agentic review mode on main, AI-generated findings stream as raw text into a side panel with no structured navigation. Users need to triage AI findings at keyboard speed — accept good ones as annotations, dismiss false positives, edit imprecise suggestions. This is the missing link between AI-generated analysis and mdiff's structured annotation system. Directly requested by repo owner in Issue #71.
**Scope**: AutoReviewState with ReviewFinding entries, auto-review panel component, accept/dismiss/edit/jump-to-code actions, filter cycling, finding count summary. Builds on existing AgenticReviewComplete action.
**Competitive reference**: Claude Code Review severity-ranked findings, Cursor's accept/reject implicit feedback pattern.

#### Diff Timeline Scrubber (NEW)
**Status**: Spec written (033), Cursor agent launched (2026-03-27)
**Rationale**: Coding agents produce multiple commits per session — iterating on code, fixing errors, adding tests. The final diff flattens all iterations, making it impossible to understand the agent's reasoning process. vibetracer (Rust, released March 2026) validates this pattern with timeline-based diff replay. mdiff's existing attribution system maps hunks to commits; the timeline scrubber extends this by allowing users to isolate individual commits and see the diff at each step.
**Scope**: TimelineState, horizontal timeline bar component, commit scrubbing with < and > keys, single-commit diff computation (parent..commit), combined diff caching, navigator file list filtering per commit.
**Competitive reference**: vibetracer (Rust crate for AI edit timeline replay), git-time-machine (TUI reflog navigation).

#### Inline Diff Minimap (Promoted from P2)
**Status**: Spec written (034), Cursor agent launched (2026-03-27)
**Rationale**: When reviewing large files (500+ lines) modified by coding agents, reviewers lose orientation. VS Code's minimap is one of its most-used navigation features. No TUI diff viewer currently offers this. Promoted from P2 based on growing file sizes in agent output and validation from competitive landscape (patchcast, VS Code).
**Scope**: 3-column vertical minimap on right edge of diff view, color-coded by change type (green/red/yellow), current viewport indicator, density calculation from display map, M key toggle.
**Competitive reference**: VS Code minimap, patchcast visual diff approach.

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
**Status**: **Promoted to P1** (2026-03-27)
**Rationale**: Vertical minimap on the right edge of the diff view showing change density across the file. Color-coded bar indicating add/delete regions for quick visual orientation. Inspired by VS Code's minimap and patchcast's visual diff approach. Helps reviewers understand where changes are concentrated without scrolling.
**Scope**: Render a narrow column (2-3 chars wide) on the right edge of the diff view. Map each row to a proportional section of the file. Color green for additions, red for deletions, dim for context. Highlight current viewport position.

#### Annotation Export to GitHub PR Comments
**Status**: Promoted from P3 (2026-03-17)
**Rationale**: One-key export that posts annotations as GitHub PR review comments. Maps mdiff categories/severities to GitHub review format. Closes the loop: review in mdiff -> feedback on GitHub. Research confirms PR comments are the standard AI agent input format (GitHub Copilot coding agent consumes PR review comments to iterate). This is the missing link in mdiff's feedback pipeline.
**Scope**: GitHub API integration, map annotation categories to review comment format, batch submission as a single review, support for line-specific comments.

#### Agent Session Transcript Viewer
**Status**: NEW (2026-03-18)
**Rationale**: When reviewing agent output, the "why" is as important as the "what." Currently, reviewers must leave mdiff to read agent session logs. A side panel showing the AI agent's reasoning/conversation alongside the diff — linked to specific files/hunks the agent modified — would eliminate this context switch. Inspired by carn (Go TUI for browsing Claude/Codex sessions) and Ralph TUI (agent orchestrator with subagent tracing). Complements the Ask a Question feature (#52) but is passive (read existing logs) rather than active (ask new questions).
**Scope**: Parse agent session formats (Claude JSONL, Codex markdown logs). Side panel toggled with a keybinding. Link log entries to files/hunks where possible. Scrollable, searchable transcript view.

#### Custom Review Rubric File (`.mdiff-review.md`)
**Status**: NEW (2026-03-18)
**Rationale**: mdiff's annotation categories and review checklists are currently generic. Projects have specific review criteria (e.g., "always check for SQL injection in DB files," "verify all new API endpoints have auth middleware"). A per-project `.mdiff-review.md` or `.mdiff.toml` file could define custom annotation categories, project-specific checklist items, required review criteria, and custom severity definitions. Inspired by the CLAUDE.md/AGENTS.md/copilot-instructions.md ecosystem and GitHub Copilot's custom instructions mechanism.
**Scope**: File discovery and parsing, extend annotation categories, populate checklist from project config, display project-specific review guidance.

#### Review Session Persistence & Resume
**Status**: NEW (2026-03-18)
**Rationale**: Large agent changesets often cannot be reviewed in a single session. Currently, mdiff persists annotations but not the full review context (scroll position, reviewed files, search history, current file). Named review sessions with full state serialization would allow reviewers to resume exactly where they left off. Inspired by carn's session persistence and Ralph TUI's session management.
**Scope**: Serialize full AppState to disk (JSON or binary), named sessions, resume command (`mdiff --resume <session>`), auto-save on quit, session listing.

#### Pluggable Diff Backend System
**Status**: New (2026-03-19)
**Rationale**: ftdv and lazygit demonstrate the value of pluggable diff renderers. Users should be able to pipe diff output through their preferred tool (delta, difftastic, bat) for custom syntax highlighting and formatting. This is a key extensibility pattern that competitive tools already offer.
**Scope**: Add `[diff_backend]` config section to `mdiff.toml` with template variables for external diff commands. Capture ANSI output and render in diff view. Fall back to built-in renderer when no backend configured.
**Competitive reference**: ftdv (multiple backends), lazygit (template variables for diff tools).

#### Diff Chunking / Smart Segmentation
**Status**: New (2026-03-19)
**Rationale**: Code review research shows defect detection drops sharply past 400 lines. Large agent changesets often exceed this threshold per file. Auto-segmenting diffs into reviewable chunks with progress tracking would improve review quality. Extends the Review Progress Bar (022) concept to within-file granularity.
**Scope**: Heuristic-based chunking at function/block boundaries (using tree-sitter), chunk navigation with `Ctrl+]`/`Ctrl+[`, chunk-level progress tracking, configurable chunk size threshold.
**Research basis**: PRs under 400 lines get 75%+ defect detection; each additional 100 lines adds ~25 minutes of review time.

#### Review Session Bookmarks
**Status**: Spec written (028), Cursor agent launched (2026-03-23)
**Rationale**: Single-key bookmark (b) reduces "flag and continue" friction from ~5 keystrokes to 1. Named bookmarks (m + letter) enable vim-style cross-file references.
**Scope**: b toggle, B list panel, ]b/[b navigation, m+letter named bookmarks, gutter diamond markers, session persistence.

#### Commit Split Suggestions
**Status**: New (2026-03-23)
**Rationale**: Agents produce monolithic commits. Suggesting logical commit boundaries helps users create cleaner git history. Inspired by oh-my-pi.
**Scope**: Heuristic grouping by directory/feature, suggestion UI, one-key split commit action.

#### Terminal Compatibility / Graceful Color Fallback
**Status**: New (2026-03-23)
**Rationale**: Codex has illegible diffs on terminals without true color (bug #15416). mdiff should detect and gracefully degrade.
**Scope**: Detect color support, fallback palettes, ensure legibility.

#### Synchronized Output (Flicker-Free Rendering)
**Status**: New (2026-03-23)
**Rationale**: Claude Code has TUI flickering (bug #37283). DECSET 2026 synchronized output fixes this.
**Scope**: Enable synchronized output in crossterm backend.

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

#### Batch File Operations (Multi-Select)
**Status**: NEW (2026-03-18)
**Rationale**: When reviewing large changesets, reviewers often want to perform the same operation on multiple files: mark a directory of test files as reviewed, stage all config changes, or apply the same annotation to related files. Currently each operation is per-file. Multi-select in the navigator (visual mode with `V`) with batch actions would significantly speed up review of large changesets. Inspired by lazygit's multi-select and Yazi's batch operations.
**Scope**: Multi-select state in NavigatorState, visual mode toggle in navigator, batch mark-reviewed, batch stage, batch annotate.

#### Diff Blame Integration
**Status**: NEW (2026-03-18)
**Rationale**: When reviewing agent-modified code, it is useful to know which surrounding context lines were written by humans vs. by previous agent runs. A blame overlay (toggled with `B`) showing author and timestamp for context lines helps reviewers calibrate trust — human-authored code that the agent modified deserves more scrutiny than agent-authored code being refactored.
**Scope**: Git blame parsing for the current file, overlay in diff view gutter, toggle keybinding, author coloring.

#### Change Impact Graph
**Status**: NEW (2026-03-18)
**Rationale**: When a coding agent modifies files across a codebase, understanding which changes are connected (via imports, function calls, type dependencies) helps reviewers assess blast radius. A simple dependency visualization showing which changed files import from each other would help prioritize review order. Feasibility is lower due to language-specific import parsing requirements.
**Scope**: Import/dependency graph extraction (Rust `use`, JS `import`, Python `import`), visualization as a simple ASCII graph or tree, highlight downstream files when viewing a change.

#### Specialized Review Passes
**Status**: New (2026-03-19)
**Rationale**: Claude Code Review's multi-agent architecture dispatches specialized parallel reviewers (bugs, security, compliance, git context). A similar UX pattern could guide human reviewers through focused passes rather than reviewing everything at once — logic pass, security pass, style pass, test pass.
**Scope**: Review pass selector UI, per-pass annotation state tracking, pass-specific checklist templates, pass completion indicators.
**Competitive reference**: Claude Code Review multi-agent architecture, Archbot agentic review loops.

#### Diff Rendering Quality Indicators
**Status**: New (2026-03-23)
**Rationale**: Review fatigue warnings when diff exceeds 400-line research threshold.
**Scope**: Per-file size warnings, configurable thresholds.

#### Commit-Level Annotation Export (Git Notes)
**Status**: New (2026-03-23)
**Rationale**: Export annotations as git notes attached to commits for persistent audit trail.
**Scope**: Export as git notes, attach to SHAs, read back when loading diffs.

---

## Open Issues Triage

| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #71 | Feature | P1 | Open | **NEW** — Spec 032 written, Cursor agent launched (2026-03-27). Automated review surface for AI findings. |
| #70 | Feature | P1 | Open — Implemented on main | Agentic review mode landed. Foundation for #71. |
| #65 | Bug | P0 | Open | Spec 030 written, Cursor agent finished (2026-03-24). PR #67 open. File preview on hover broken in tree view. |
| #64 | Bug | P0 | Open | Spec 029 written, Cursor agent finished (2026-03-24). PR #72 open. Tree view has 5 inconsistencies. |
| #63 | UX | P1 | Open | Spec 031 written, Cursor agent finished (2026-03-24). PR #69 open. Go-to-line and fuzzy file picker. |
| #53 | Feature | P1 | Open — PR #56 merged (2026-03-20) | **Implemented** — File Tree Navigator merged. Issue should be closed. |
| #52 | Feature | P1 | Open | Spec 021 written, Cursor agent finished but no PR. Needs re-launch. |
| #37 | Feature | P1 | Open (PR #45 merged, issue not closed) | Implementation complete, issue should be closed |
| #35 | Bug | P0 | Open (PR #51 merged) | Fix merged, issue should be closed |
| #25 | Bug | P0 | Open — fix from PR #50 insufficient | **Second pass** — Spec 035 written, Cursor agent launched (2026-03-30). Scroll still clips bottom lines + adding boundary padding. |
| #38 | UX/Safety | P0 | Closed | Resolved |

---

## Competitive Landscape

### Tools Tracked
- **OpenDiffs** (Node.js CLI): NEW (2026-03-22). AI-powered structured code review of staged git diffs using Claude Code or Codex agents. Produces scored markdown reviews. Validates mdiff's problem space from an AI-first angle.
- **critique** (TypeScript/Bun): TUI diff viewer with watch mode, glob filtering, word-level diff, tree-sitter syntax highlighting. v0.1.129 released 2026-03-17. 1,091 stars, 38 releases, rapid iteration. Key competitive features to track.
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
- **Anduin** (Rust + Iced): Brand-new Git GUI specifically designed for coding-agent workflows and worktrees. Created March 16, 2026. Rust-based like mdiff but uses a GUI (Iced) rather than TUI. Directly overlaps with mdiff's target use case from a GUI angle.
- **carn** (Go TUI): New TUI for browsing Claude and Codex AI coding sessions. Features diff display with colored additions/removals, markdown rendering with syntax highlighting, and full-text search across session transcripts. Targets the agent session review use case.
- **Ralph TUI**: Open-source terminal UI orchestrator connecting multiple AI coding agents to task trackers. Runs autonomous agent loops with subagent tracing and session persistence. Represents the orchestration layer pattern.
- **ftdv** (Rust/ratatui): File Tree Diff Viewer combining diffnav navigation with lazygit's flexible diff tool configuration. Features: vim-style nav, interactive file tree with directory folding, review tracking checkboxes, real-time search filtering, persistent state, ANSI color support, pluggable diff backends (delta, bat, ydiff, difftastic). Direct competitor to mdiff with overlapping features.
- **keifu** (Rust): New TUI for Git commit graph visualization with colored branch graphs, commit detail panels, changed file stats, branch search. Adjacent tool in the Rust Git TUI space.
- **dead-ringer** (Rust): Interactive binary diff viewer with keyboard navigation and color highlighting. Part of broader terminal diff ecosystem.
- **cmux** (macOS terminal): Ghostty-based terminal gained 7.7k stars in first month. AI agent orchestration focus with vertical tabs. Demonstrates massive appetite for specialized terminal tools.
- **Muse/Manyana**: VCS with conflict-aware diff views labeling each hunk with action and side. Inspired Agent Attribution Labels.
- **oh-my-pi**: AI terminal agent with agentic git tools and atomic commit splitting.
- **Superset**: Desktop workspace for multi-agent orchestration with built-in diff review.
- **Orchestrator**: tmux-style split panes for parallel Claude Code sessions.
- **Kaleidoscope**: macOS-native diff/merge tool. Excellent visual polish but no TUI, no agent integration.
- **lazygit**: Gold standard for TUI git UX. Hunk-level operations, keyboard-driven workflow. Part of "Terminal Power Trio" (Ghostty + Yazi + Lazygit).
- **fzf**: Gold standard for fuzzy search UX in terminals.
- **jjui**: TUI for jujutsu (jj), panel-based keyboard-driven paradigm.
- **gstack** (CLI): Garry Tan's open-source Claude Code toolkit with dedicated `/review` mode for production risk assessment and code review.
- **vibetracer** (Rust): TUI tool for tracing and replaying AI coding assistant edits with timeline-based diff viewer. Cross-agent tracing for Claude Code, Cursor, Codex CLI. Exports to git-compatible formats. Direct overlap with mdiff's focus on agent diff review.
- **OpenClaw TUI**: Terminal AI agent with slash commands and shortcuts. Compared alongside Claude Code and OpenCode as competing terminal-based AI developer tools.

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

### Coding Agent Review Ecosystem & Session Browsing (from 2026-03-18 research)
- **Anduin (Rust + Iced)**: Brand-new Git GUI specifically designed for coding-agent workflows and worktrees. Created March 16, 2026. Uses Rust + Iced (GUI, not TUI). Directly validates mdiff's target use case — the ecosystem is converging on "tools specifically for reviewing coding agent output."
- **carn (Go TUI)**: New TUI (v0.1.0, March 17) for browsing Claude and Codex AI coding sessions. Features diff display, markdown rendering with syntax highlighting, and full-text search across session transcripts. KEY INSIGHT: Session transcript browsing alongside diffs is a pattern mdiff should adopt (inspired Agent Session Transcript Viewer idea).
- **Ralph TUI**: Open-source terminal UI orchestrator connecting multiple AI coding agents to task trackers. Autonomous agent loops with subagent tracing and session persistence. Represents the orchestration layer pattern — mdiff could display agent traces/reasoning.
- **critique v0.1.129**: Updated to 1,091 stars (up from 1,087 in 2 days), 38 releases. Competitive pressure continues to increase.
- **Anthropic Claude Code Review multi-agent architecture**: Dispatches specialized parallel reviewers per PR, each focusing on a different failure mode. Validates mdiff's structured annotation category approach.
- **Human-in-the-loop as permanent architecture**: Industry consensus shifting from "temporary scaffold" to "genuine partnership." mdiff should optimize for permanent, not temporary, review workflows.
- **Custom instructions for AI code review (Copilot pattern)**: Natural language project-specific review criteria. Inspired Custom Review Rubric File idea.
- **78% of production sites use AI-assisted development; ~30% of dev time on code reviews**: Review efficiency is THE bottleneck — strongest market validation yet for mdiff's mission.
- **GitTop (Go TUI)**: Git stats dashboard built with Bubble Tea. Activity heatmaps and contributor analytics. Heatmap visualization pattern for change activity.

### Competitive Analysis & Code Review Research (from 2026-03-19 research)
- **ftdv (Rust/ratatui)**: Direct competitor discovered — combines diffnav navigation with lazygit's diff tool config. Has review tracking checkboxes AND pluggable diff backends (delta, bat, difftastic). Two key competitive features mdiff should consider: pluggable backends for extensibility and the checkbox review tracking UX.
- **keifu (Rust TUI)**: New Git commit graph visualization tool. Validates continued growth in Rust Git TUI tooling.
- **cmux (Ghostty-based terminal)**: Gained 7.7k GitHub stars in first month with AI agent orchestration focus. Shows massive appetite for specialized terminal tools in the AI workflow space.
- **skim v4.0.0**: Major release of Rust fuzzy finder (6,682 stars). Benchmark for Rust TUI tool success.
- **Code review research (2026)**: PRs under 400 lines get 75%+ defect detection; each additional 100 lines adds ~25 minutes. Large diffs (400+ lines) sharply degrade human defect detection. Implication: mdiff needs diff chunking/segmentation for large files.
- **Claude Code Review multi-agent**: 5 specialized parallel reviewers per PR, <1% incorrect finding rate by focusing on logic over style. Structured categories map directly to mdiff's annotation system.
- **CodeScout (RL for code agents)**: Demonstrates RL reward design for code-related tasks. Relevant to future RLHF signal design from mdiff feedback.
- **Technical debt increases 30-41% after AI adoption**: Code churn expected to double in 2026. Review tools are THE bottleneck.
- **dead-ringer (Rust)**: Binary diff viewer. Broader diff ecosystem competitors tracked: difftastic, delta, diff-so-fancy, icdiff, ydiff, diffr.

### TUI Resurgence & Agent Tool Ecosystem (from 2026-03-23 research)
- TUI resurgence confirmed (Leon Mika blog) driven by AI coding agents. Validates mdiff approach.
- Muse/Manyana conflict-aware diffs inspired Agent Attribution Labels (spec 026).
- oh-my-pi agentic git tools inspired Commit Split Suggestions.
- Codex diff rendering bug (#15416) inspired Terminal Compatibility feature.
- Claude Code TUI flickering (#37283) inspired Synchronized Output feature.
- Agent-RLVR (Scale Labs): Structured guidance improves agent performance 9.4% to 22.4%. Inspired Agent Guidance Export (spec 027).
- Superset, Orchestrator: competing multi-agent tools tracked.

### OpenDiffs & Agent Feedback Patterns (from 2026-03-24 research)
- **OpenDiffs** (released March 22, 2026): CLI tool that reviews staged git diffs using Claude Code or Codex agents, producing scored markdown reviews organized by branch.
- **Agent-RLVR** (Scale Labs): Structured guidance improves agent task completion 9.4% to 22.4%. Validates mdiff's Agent Guidance Export (spec 027).
- **Rich Feedback RL**: Field converging from scalar reward RL toward critiques, comparisons, checklists, verifier signals.
- **git-time-machine** (Rust/ratatui): New TUI tool for visual git reflog navigation with vim keybindings.
- **GitHub trending March 2026**: Dominated by AI agent frameworks, no TUI/diff tools in top trending — mdiff occupies a niche with growing demand.

### Implicit Feedback & Timeline Patterns (from 2026-03-27 research)
- **Cursor real-time RL**: Uses implicit user actions (accept/reject/edit) from production coding sessions as reward signals, shipping improved models every 5 hours. Low-friction feedback capture more valuable than elaborate forms. Validates mdiff's quick-reaction scoring approach.
- **Scale AI Online Rubrics**: Dynamically generated evaluation criteria from pairwise comparisons outperform static rubrics by 8%. Model for evolving annotation schemas.
- **vibetracer (Rust)**: Timeline-based scrubbing through AI agent edit history. Cross-agent tracing. Validates the Diff Timeline Scrubber feature.
- **Reddit r/opencodeCLI**: Terminal-only developer explicitly requesting TUI side-by-side diff viewer for reviewing agent output. Rejects Cursor and Neovim. Direct validation of mdiff's value proposition.
- **HN "slowing the fuck down" (1074 points)**: Growing concern about AI codebases being incomprehensible. Developer review tooling is essential infrastructure, not optional.
- **Claude Code Review adaptive depth**: Scales review intensity with PR size — lightweight for <50 lines, deeper multi-agent for 1000+. Pattern for adaptive review guidance in mdiff.

---

## Changelog

### 2026-03-24
- **NEW SPEC 029**: Tree View Inconsistencies (Issue #64) — 5 bug fixes for tree navigator
- **NEW SPEC 030**: File Preview on Hover (Issue #65) — sync diff pane with tree cursor
- **NEW SPEC 031**: Go-to-Line & Fuzzy File Picker (Issue #63) — vim command bar + Ctrl+P file search
- **Promoted Issues #64, #65 to P0**: Owner-filed tree view regressions
- **Promoted Issue #63 to P1**: Owner-filed UX improvement for standard navigation shortcuts
- **Cursor agents launched**: specs 029, 030, 031
- **PR Review**: PR #62 (Agent Attribution Labels), PR #61 (Agent Guidance Export, draft), PR #55 (Annotation Suggestions) reviewed
- **Research**: OpenDiffs (AI-powered diff review), Agent-RLVR (structured guidance), git-time-machine (Rust/ratatui)
- Added OpenDiffs to competitive landscape

### 2026-03-23
- **ROADMAP.md recovered** from corruption (sandbox path reference replaced full content)
- **NEW SPEC 026**: Agent Attribution Labels — hunk-level agent session markers
- **NEW SPEC 027**: Agent Guidance Export — structured feedback formats for coding agents
- **NEW SPEC 028**: Review Session Bookmarks — single-key line markers
- **NEW P2**: Commit Split Suggestions, Terminal Compatibility, Synchronized Output
- **NEW P3**: Diff Rendering Quality Indicators, Commit-Level Annotation Export
- **PR Review**: PR #55 (Annotation Suggestions, +911/-66), PR #57 (Open-in-Editor, +197/-0) reviewed
- **Cursor agents launched**: specs 026, 027, 028
- Added Muse/Manyana, oh-my-pi, Superset, Orchestrator to competitive landscape

### 2026-03-20
- **NEW SPEC 025**: Quick Apply Suggestion
- **Cursor agents launched**: File Tree Navigator (PR #56), Annotation Suggestions (PR #55), Open-in-Editor (PR #57)
- **PR #56 merged**: File Tree Navigator complete

### 2026-03-19
- **NEW P2**: Pluggable Diff Backend System — configure external diff renderers (delta, difftastic, bat) via config, inspired by ftdv and lazygit patterns
- **NEW P2**: Diff Chunking / Smart Segmentation — auto-segment large diffs at function boundaries for improved defect detection, research-backed (400-line threshold)
- **NEW P3**: Specialized Review Passes — guided focused review passes (logic, security, style, tests), inspired by Claude Code Review multi-agent architecture
- **Cursor agents launched** (5th attempt): File Tree Navigator (spec 020, Issue #53), Diff Statistics Dashboard (spec 019), Ask a Question (spec 021, Issue #52)
- **Competitive discovery**: ftdv (ratatui-based TUI diff viewer with pluggable backends and review checkboxes) — direct competitor with overlapping features
- **Research**: ftdv competitive analysis, keifu (Rust Git TUI), cmux (7.7k stars terminal), skim v4.0.0, code review research (400-line defect detection threshold), Claude Code Review multi-agent, CodeScout RL, technical debt +30-41% post-AI
- Added ftdv, keifu, dead-ringer, cmux to competitive landscape
- Added "Competitive Analysis & Code Review Research" research section
- PR #54 (release v0.1.19) reviewed — version bump only, no action needed
- Open issues unchanged: #53, #52 awaiting implementation; #37, #35, #25 have fixes merged and need manual closure

### 2026-03-18
- **NEW SPEC**: Review Progress Bar (022) — persistent visual indicator of review completion in navigator footer, block-character progress bar, count and percentage display
- **NEW P2**: Agent Session Transcript Viewer — display agent reasoning/conversation alongside diffs, inspired by carn and Ralph TUI
- **NEW P2**: Custom Review Rubric File (`.mdiff-review.md`) — project-specific annotation categories, checklist items, and review criteria
- **NEW P2**: Review Session Persistence & Resume — named review sessions with full state serialization for multi-session reviews
- **NEW P3**: Batch File Operations (Multi-Select) — multi-select in navigator with batch mark-reviewed, stage, annotate
- **NEW P3**: Diff Blame Integration — git blame overlay showing author/timestamp for context lines, distinguishing human vs agent code
- **NEW P3**: Change Impact Graph — dependency visualization for changed files to assess blast radius
- **Cursor agents launched** (3rd attempt): File Tree Navigator (spec 020, Issue #53), Diff Statistics Dashboard (spec 019), Review Progress Bar (spec 022)
- **Discovered**: Split Diff View already exists in codebase (DiffViewMode::Split, render_split, ToggleViewMode action) — removed from ideation
- **Research**: Anduin (Rust Git GUI for agent workflows), carn (Go TUI for AI sessions), Ralph TUI (agent orchestrator), critique v0.1.129 (1,091 stars), Anthropic multi-agent code review, human-in-the-loop as permanent architecture, custom review instructions pattern, 78% AI-assisted development stat
- Added Anduin, carn, Ralph TUI to competitive landscape
- Updated critique entry (v0.1.129, 1,091 stars, 38 releases)
- Added "Coding Agent Review Ecosystem & Session Browsing" research section
- PR #54 (release v0.1.19) remains the only open non-feature PR

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
