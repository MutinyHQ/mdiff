# mdiff Roadmap & Feature Backlog

> **Maintained by**: Milo (automated ideation agent)
> **Last updated**: 2026-03-05
> **Schedule**: New ideas are evaluated and prioritized every 24 hours

This document tracks feature ideas, prioritized issues, and the rationale behind them. It draws from competitive analysis of tools like **critique**, **tuicr**, **acre**, **difi**, **git-review**, **Kaleidoscope**, **lazygit**, **fzf**, and patterns from RLHF/human feedback research.

---

## Priority Tiers

### P0 — Critical Path (Active PRs)

#### 1. Hunk-Level Navigation (`]` / `[` keys)
**Status**: PR open
**Rationale**: Every serious diff tool (vim-fugitive, lazygit, tig, delta) supports jumping between hunks. Currently mdiff only has line-by-line (`j`/`k`) and page scrolling. For large agent-generated diffs with hundreds of lines, reviewers need to skip context and jump between actual changes.
**Scope**: Add `JumpNextHunk` / `JumpPrevHunk` actions, remap existing `]`/`[` annotation navigation to `Ctrl+]`/`Ctrl+[`, display "Hunk N/M" in the HUD, support cross-file hunk jumping.
**Competitive reference**: vim `[c`/`]c`, lazygit hunk navigation, tuicr `{`/`}` keys.

#### 2. Annotation Categories & Severity Levels
**Status**: PR open
**Rationale**: Structured feedback is dramatically more effective than free-text when communicating with coding agents. Research on RLHF shows categorized, severity-tagged annotations lead to better model responses. This transforms mdiff from a "leave comments" tool into a structured feedback system.
**Scope**: Add category picker (Bug/Style/Performance/Security/Suggestion/Question/Nitpick) and severity levels (Critical/Major/Minor/Info) to the annotation flow. Single-keypress selection. Update prompt template to include structured metadata. Backward-compatible with existing sessions.
**Competitive reference**: GitHub PR review comment types, V7 Labs RLHF annotation categories, Linear issue priority levels.

#### 3. Global Fuzzy Search Across All Diff Content (`Ctrl+F`)
**Status**: PR open
**Rationale**: The existing `/` search only filters file names in the navigator. When reviewing large agent changesets spanning 20+ files, you need to find specific code patterns, variable names, or string literals across ALL changed files. The `nucleo` crate is already in Cargo.toml but only used for file name filtering.
**Scope**: Add `Ctrl+F` for global content search across all diff hunks, using nucleo for fuzzy matching with an exact-match toggle (`Ctrl+G`). Live incremental results, match highlighting in diff view, "Match N/M" counter, cross-file navigation with `n`/`N`.
**Competitive reference**: fzf search patterns, VS Code Ctrl+Shift+F, Kaleidoscope full-text search.

---

### P1 — High Impact

#### 4. File Tree Navigator with Directory Grouping
**Rationale**: When agents modify 30+ files across multiple directories, the flat file list becomes unwieldy. A collapsible tree view grouped by directory (like VS Code's explorer) would make navigation much faster.
**Scope**: Add tree view mode to the navigator (toggle with `T`), collapse/expand directories, show file counts per directory, sort by path or by change type.
**Competitive reference**: VS Code file explorer, GitHub PR file tree, lazygit file tree.

#### 5. Diff Statistics Dashboard
**Rationale**: Before diving into line-by-line review, reviewers need an overview: how many files changed, total additions/deletions, which files have the most churn. This is especially important for agent output where you want to quickly assess if the agent went off track.
**Scope**: Add a summary view (toggle with `S`) showing: total files/additions/deletions, per-file sparkline bars, file type breakdown, largest files by change size, annotation coverage stats.
**Competitive reference**: GitHub PR stats bar, `git diff --stat`, diffray summary.

#### 6. Configurable Keybinding System
**Rationale**: Power users expect to customize keybindings. The current hardcoded mapping in `event.rs` works but doesn't allow personalization. A config-driven system would let users remap any key and create custom workflows.
**Scope**: Add `[keybindings]` section to `~/.config/mdiff/config.toml`, support conditional bindings (different behavior per context), vim-mode awareness, document all bindable actions.
**Competitive reference**: Atuin's conditional keybinding system, lazygit custom keybindings, neovim keymaps.

#### 7. Review Progress Tracking with Completion Percentage
**Rationale**: The existing review state tracks per-file reviewed/unreviewed, but doesn't show overall progress. For large reviews, knowing "I've reviewed 12/34 files (35%)" with a progress bar is essential for staying motivated and tracking sessions.
**Scope**: Add progress bar to the HUD, percentage complete, time tracking per file, estimated remaining time, ability to export review summary.
**Competitive reference**: tuicr review tracking, GitHub PR review progress, Reviewable progress bars.

#### 8. Annotation Quick-Reactions (Single-Key Line Scoring)
**Rationale**: Full annotation flow (select → category → severity → comment) is powerful but heavy for rapid feedback. RLHF research shows that scalar ratings alongside categorical feedback dramatically improves signal quality. Quick-reactions let reviewers score individual lines or hunks with a single keypress (1-5 scale) without entering any text, enabling rapid per-line quality scoring for agent feedback loops.
**Scope**: Press `1`-`5` on any diff line (or visual selection) to leave a quick score. Scores render as colored dots in the gutter (red→green gradient). Included in prompt output as `[Score: 3/5]` prefix. Dashboard shows score distribution. Compatible with full annotations (score + comment). Persist in session file alongside annotations.
**Competitive reference**: RLHF scalar reward signals, Likert scales in annotation tools, GitHub emoji reactions.

#### 9. Agent Feedback Summary View
**Rationale**: After leaving annotations and scores, reviewers need to see the aggregate picture before sending feedback to agents. A summary view answers: "What did I flag? How severe was it? Is this agent improving?" Currently annotations are scattered across files with no aggregate view. This closes the feedback loop.
**Scope**: Toggle with `F` — dedicated panel showing: annotation count by category/severity, score distribution histogram, per-file annotation density, one-click structured JSON export for agent consumption, trend comparison across sessions (if multiple sessions exist for the same worktree).
**Competitive reference**: GitHub PR review summary, RLHF annotation dashboards, Linear project analytics.

#### 10. Contextual Help Overlay (Which-Key)
**Rationale**: mdiff has 50+ keybindings across 8+ contexts (navigator, diff view, visual mode, comment editor, etc.). Discovery is a major UX challenge — the HUD toggle (`?`) shows a static list, but power users need contextual hints. Lazygit, neovim (which-key), and Helix all show available keys for the current context. Every new feature we add increases the discoverability problem.
**Scope**: After a brief idle period (300ms) with no keypress following a "leader" key, show a floating overlay listing available follow-up keys and their actions for the current context. Covers all focus panels and modal states. Dismisses on any keypress. Can be disabled in config. Shows key + short description in a compact grid.
**Competitive reference**: neovim which-key.nvim, Helix context help, Emacs which-key, lazygit contextual hints.

---

### P2 — Medium Impact

#### 11. Inline AI Suggestions (Agent-in-the-Loop)
**Rationale**: Instead of just sending feedback TO agents, mdiff could also receive suggestions FROM agents during review. Imagine selecting a problematic section, pressing a key, and getting an inline AI suggestion for how to fix it — without leaving the diff view.
**Scope**: Add `Ctrl+S` to request an AI suggestion for the selected code, display inline suggestion in a overlay panel, accept/reject/modify the suggestion, integrate with configured agents.

#### 12. Multi-Worktree Comparison View
**Rationale**: When multiple agents work on the same problem in parallel worktrees, you want to compare their approaches side by side. Currently you can only view one worktree at a time.
**Scope**: Add a comparison mode that shows two worktree diffs side by side, highlight differences between agent approaches, allow cherry-picking hunks from either side.

#### 13. Bookmark System for Large Reviews
**Rationale**: In multi-session reviews spanning hours, you need to bookmark positions to return to later. Annotations serve a different purpose (feedback for agents). Bookmarks are for the reviewer's own navigation.
**Scope**: Add `b` to toggle a bookmark on the current line, `B` to open bookmark list, bookmarks persist per session, show bookmark indicators in the gutter.

#### 14. Word-Level (Intra-Line) Diff Highlighting
**Rationale**: Currently the diff shows entire lines as added/deleted. For modified lines, highlighting the specific words or characters that changed within a line makes review much faster.
**Scope**: Compute word-level diff for paired add/delete lines, highlight changed words with a brighter color, support character-level granularity toggle.
**Competitive reference**: critique word-level diffs, GitHub word-level highlighting, delta intra-line highlights.

#### 15. Git Blame Integration
**Rationale**: When reviewing agent changes, it's useful to see the blame context — who wrote the code being modified and when. This helps assess risk and understand the codebase better.
**Scope**: Add `B` to toggle blame view on the old side, show author/date/commit for each line, click to expand blame details.

#### 16. Smart Review Ordering (Auto-Sort Files by Priority)
**Rationale**: Agent changesets often touch 20-50+ files spanning tests, implementation, config, and docs. Reviewing in alphabetical or git order is suboptimal. Research on code review effectiveness shows that reviewing tests first (to understand intent), then core implementation, then peripheral files leads to better bug detection. Flat file lists bury the most important files.
**Scope**: Add sort modes toggled with `O`: by change size (largest first), by file type priority (tests → impl → config → docs), by annotation density, by review status. Show sort indicator in navigator header. Remember sort preference in config.
**Competitive reference**: GitHub PR file ordering, Reviewable smart ordering, IntelliJ change list grouping.

#### 17. Glob-Based File Filtering
**Rationale**: The current `/` search does fuzzy matching on file names, which works for finding a specific file. But when reviewing agent output touching `*.rs`, `*.toml`, `*.md`, and `*.yaml` files, you often want to filter to just one type or exclude test files. fzf-lua's glob filtering in live grep is a beloved power-user feature for exactly this pattern.
**Scope**: Extend the navigator search (`/`) to support glob patterns when input starts with `>` (e.g., `>*.rs`, `>!*test*`, `>src/components/**`). Show "glob mode" indicator. Combine with existing fuzzy search (glob narrows, then fuzzy matches within). Leverage nucleo for performance.
**Competitive reference**: fzf-lua glob filtering, VS Code file exclude patterns, ripgrep glob flags.

#### 18. Diff Heatmap Mode (Change Density Visualization)
**Rationale**: In the navigator, all files look the same regardless of how much they changed. In the diff view, it's hard to see where changes cluster in a large file. A heatmap provides instant visual signal about where to focus attention — critical when triaging large agent changesets.
**Scope**: Color-code files in the navigator by change density (red=heavy, yellow=moderate, green=minor). Add a scrollbar minimap in the diff view showing change density along the file height. Toggle with `H`. Colors configurable in theme.
**Competitive reference**: VS Code minimap change indicators, Sublime Text minimap, GitHub contribution heatmap.

---

### P3 — Nice to Have

#### 19. Session Export to Markdown/JSON
Export the complete review session (annotations, diff stats, review progress) to a structured format for sharing or archival.

#### 20. Notification on Agent Completion
Watch for agent process completion in worktrees and show a notification in the TUI.

#### 21. Custom Prompt Templates
Allow users to define custom prompt template formats in config.toml for different agent types.

#### 22. Image Diff Support
For changes to image files, show a visual comparison (pixel diff, side-by-side, onion skin).

#### 23. Split Pane for Multiple Files
View two different files' diffs simultaneously in a horizontal or vertical split.

#### 24. Undo/Redo for Git Operations
Add undo capability for stage/unstage/restore operations to prevent accidental data loss.

#### 25. Diff Filter by Change Type
Filter the file list to show only additions, deletions, modifications, or renames.

#### 26. Integration with GitHub/GitLab PR Workflows
Push annotations as PR review comments directly from mdiff.

#### 27. Command Transparency Log
**Rationale**: Lazygit's most-loved community feature is showing the underlying git commands it runs. This builds trust, helps users learn git, and aids debugging. mdiff performs git operations (stage, unstage, restore, commit) but the user never sees what's happening underneath.
**Scope**: Floating log panel toggled with `~` showing recent git commands and their stdout/stderr output. Scrollable history. Commands timestamped.

#### 28. External Editor Integration for Long Annotations
**Rationale**: The TUI text input is fine for short comments but limiting for detailed architectural feedback. The standard Unix pattern (used by git commit, crontab -e, etc.) is to open `$EDITOR` for long-form text.
**Scope**: Press `E` in the comment editor to open the current annotation text in `$EDITOR`. On editor close, import the text back. Supports multi-paragraph feedback with proper formatting.

---

## Competitive Landscape Notes

### Tools Analyzed
- **critique** (1060 stars): TypeScript/Bun TUI, syntax highlighting, split view, word-level diffs, watch mode. Strong UX but no agent integration.
- **tuicr**: Rust TUI for AI-generated diff review, GitHub PR-style infinite scroll, vim keybindings, clipboard export as structured Markdown. Closest competitor to mdiff's vision.
- **acre**: Python TUI for AI-assisted collaborative code review with Claude. Real-time collaboration model is interesting but different use case.
- **difi**: Go TUI with Neovim integration, recently posted on Hacker News. Community feedback emphasized importance of high-quality screenshots/GIFs for TUI tool adoption.
- **git-review**: Rust TUI for interactive code review, very early stage. Uses Claude AI integration. Direct Rust-peer competitor.
- **Kaleidoscope**: macOS-native diff/merge tool. Excellent visual polish but no TUI, no agent integration.
- **lazygit**: Gold standard for TUI git UX. Hunk-level operations, keyboard-driven workflow. Community's most-loved feature: showing underlying git commands.
- **fzf**: Gold standard for fuzzy search UX in terminals. Event-action binding, scoring algorithms.
- **jjui**: TUI for jujutsu (jj), panel-based keyboard-driven paradigm for commit graph editing. Shows the lazygit UX pattern has legs beyond git.

### Key Market Gap
No tool combines all three: (1) TUI-native diff review, (2) structured human feedback collection, (3) direct integration with coding agents. mdiff is uniquely positioned here.

### UX Patterns Worth Adopting (from 2026-03-05 research)
- **Contextual keybindings per panel** (lazygit, jjui): Different keys do different things depending on which panel has focus. mdiff already does this but should formalize it.
- **Glob-based filtering in search** (fzf-lua): Users love being able to narrow results with glob patterns before fuzzy matching.
- **Command transparency** (lazygit): Showing the underlying commands builds trust and helps learning.
- **Disable dangerous operations by default** (lazygit community feedback): Force-push disabled, restore requires confirmation (mdiff already has this).

---

## Changelog

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
