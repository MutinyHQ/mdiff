# mdiff Ideation Agent Journal

## 2026-03-20 (Run #10)

### Research Findings
- **Plannotator** (3,259 stars): Major competitive discovery — open-source annotation tool specifically for reviewing AI coding agent plans and code diffs. Key patterns that directly inspire mdiff features:
  - Typed annotations: comment/suggestion/concern (not just free-form text)
  - **Code suggestions embedded in review UI** — proposed replacement code, not just comments. This is the single most impactful pattern discovered. Directly inspired the Annotation Suggestions spec (023).
  - Bidirectional scroll navigation between annotations and code
  - Quick annotation labels for faster workflow
  - Auto-save annotation drafts to prevent loss of in-progress review
  - Plan diff tracking (what changed when agent revises)
  - Hierarchical folder tree for navigating changed files
  - GUI-based (TypeScript/Electron), but UX patterns are highly relevant to TUI adaptation
- **critique v0.1.129** (1,091 stars): Updated March 17. New competitive insights:
  - Responsive split view — auto-switches to unified on narrow terminals
  - Watch mode for live diff updates
  - Click-to-open in editor (via REACT_EDITOR env var)
  - Cherry pick feature (interactive file picker for changes from other branches)
  - Web preview generation (shareable HTML diffs)
- **deff** (Rust): Launched on Hacker News in March 2026. Side-by-side TUI diff review with vim-style motions, in-diff search (/, n, N), per-file reviewed toggles, mouse support. Direct competitor with overlapping features.
- **difi** (Go): TUI git diff viewer with two-pane layout (file tree left, diff right). 'e' key to open file at exact line in vim/neovim. Directly inspired the Open-in-Editor spec (024).
- **Anthropic Code Review innovation**: Confidence scoring system (0-100) for filtering review noise — only high-confidence findings posted as GitHub comments. Color-coded severity with thresholds. Focus on logic errors over style.
- **AI-generated code review challenges (2026 research)**: Volume overwhelms human attention leading to rubber-stamping. AI code looks professional regardless of correctness. Traditional "ask the author" feedback loop doesn't apply. Need structured review processes.

### Open Issues Reviewed (5 open — unchanged)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open — filed by repo owner | Spec 020 exists, Cursor agent blocked (7th attempt) |
| #52 | Feature | P1 | Open — filed by repo owner | Spec 021 exists, Cursor agent blocked (7th attempt) |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged) | Should be closed manually |

**No new issues filed since last run.** Issues #25, #35, and #37 continue to need manual closure by repo owner — fixes have been merged for 7+ days now.

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Annotation Suggestions (Inline Code Proposals) | Plannotator, GitHub suggested changes | P1 (new) | **ADDED** — spec 023, Ctrl+S two-pane editor for proposing replacement code |
| Open-in-Editor at Line | difi, deff, lazygit, critique | P1 (new) | **ADDED** — spec 024, press `e` to jump to $EDITOR at current line |
| Confidence-Based Annotation Filtering | Anthropic Code Review | Deferred | Overlaps with existing severity levels — revisit if annotation count grows large |
| Responsive Split/Unified View Toggle | critique responsive layout | P2 candidate | Auto-switch based on terminal width — nice but low priority vs core features |
| Watch Mode (Live Diff Refresh) | critique --watch | Already P2 | Validates existing roadmap item |
| Annotation Draft Auto-Save | Plannotator | P3 candidate | Safety feature, low frequency need |
| Shareable Review Export (HTML) | critique web preview | P3 candidate | Interesting but far from core TUI mission |

### Specs Written
Two new specs this cycle:
1. `.github/specs/023-annotation-suggestions.md` — Inline code proposals via Ctrl+S two-pane editor overlay. Extends Annotation struct with `suggested_code: Option<String>`. New SuggestionState + suggestion_editor component.
2. `.github/specs/024-open-in-editor.md` — Press `e` to open file in $EDITOR at current line. Terminal editor suspension/resume, GUI editor detached spawn, auto-refresh on return.

### PRs & Agents
**Open PRs:**
- PR #54 (chore: release v0.1.19) — release automation PR, version bump only. Reviewed; no action needed.

**Cursor agents blocked (model unavailable — 7th consecutive run):**
- File Tree Navigator (spec 020, Issue #53)
- Annotation Suggestions (spec 023) — NEW
- Open-in-Editor (spec 024) — NEW

Attempted with: default model (claude-4.6-sonnet-high-thinking), explicit claude-3.5-sonnet, gpt-4o, anthropic/claude-sonnet-4-20250514, and omitting model param. All returned "Model not available or invalid." This is now a critical infrastructure issue — no Cursor agents have successfully launched for mdiff in over a week.

**Previous agent results:** All mdiff Cursor agents from earlier runs are FINISHED. No running or in-progress agents.

### Mockups Generated
- [Annotation Suggestions mockup](https://www.town.com/content/image/sd72sr1eadhdfkg73rfeze200n838r06) — Two-pane code suggestion editor overlay
- [Open-in-Editor mockup](https://www.town.com/content/image/sd74f602zvghpcafxdmxp178nx839gh2) — Editor jump with status bar feedback
- [File Tree Navigator mockup](https://www.town.com/content/image/sd7e4z1mm3p74y4095jretkjhx839hc6) — Collapsible directory tree with aggregate stats

### Roadmap Updates
- **NEW P1**: Annotation Suggestions / Inline Code Proposals (`Ctrl+S`) — spec 023
- **NEW P1**: Open-in-Editor at Line (`e` key) — spec 024
- Added Plannotator, deff, difi to competitive landscape
- Updated changelog with 2026-03-20 entry

### Key Insight This Cycle
The Plannotator discovery revealed the most impactful gap in mdiff's annotation system: the ability to propose specific replacement code, not just describe what's wrong. This is the difference between "comment" feedback and "suggestion" feedback — and for AI agents consuming structured feedback, concrete code proposals are dramatically more parseable and actionable than free-text descriptions. The Annotation Suggestions feature (spec 023) is the highest-impact new idea since the Inline Q&A feature (spec 021).

## 2026-03-19 (Run #9)

### Research Findings
- **ftdv (Rust/ratatui)**: Direct competitor discovered — File Tree Diff Viewer combining diffnav navigation with lazygit's flexible diff tool configuration. Has review tracking checkboxes AND pluggable diff backends (delta, bat, ydiff, difftastic). Two key competitive features that overlap with mdiff's roadmap: the file tree navigator (our #53) and pluggable backends (new P2 idea). Built with ratatui like mdiff.
- **keifu (Rust TUI)**: New Git commit graph visualization tool with colored branch graphs, commit detail panels, changed file stats. Adjacent tool validating continued growth in Rust Git TUI space.
- **cmux (Ghostty-based terminal)**: Gained 7.7k GitHub stars in its first month with AI agent orchestration focus. Demonstrates massive appetite for specialized terminal tools in the AI workflow space.
- **skim v4.0.0**: Major release of Rust fuzzy finder (6,682 stars). Benchmark for what a well-executed Rust TUI tool can achieve at scale.
- **Code review research (2026)**: PRs under 400 lines get 75%+ defect detection; each additional 100 lines adds ~25 minutes of review time. Large diffs (400+ lines) sharply degrade human defect detection. CRITICAL INSIGHT: mdiff needs diff chunking/segmentation for large files — inspired new P2 "Diff Chunking / Smart Segmentation" idea.
- **Claude Code Review multi-agent architecture**: 5 specialized parallel reviewers per PR (bugs, security, compliance, git context, comment verification), <1% incorrect finding rate by focusing on logic errors over style. Structured categories map directly to mdiff's annotation system. Inspired new P3 "Specialized Review Passes" idea.
- **CodeScout (RL for code agents)**: Demonstrates RL reward design for code-related tasks. Relevant to future RLHF signal design from mdiff's structured feedback output.
- **Technical debt increases 30-41% after AI tool adoption**: Code churn expected to double in 2026. Review tools are THE bottleneck — strongest ongoing market validation.
- **dead-ringer (Rust)**: Binary diff viewer. Broader diff ecosystem tracked: difftastic, delta, diff-so-fancy, icdiff, ydiff, diffr.

### Open Issues Reviewed (5 open)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open — filed by repo owner | Spec 020 exists, Cursor agent blocked (5th attempt) |
| #52 | Feature | P1 | Open — filed by repo owner | Spec 021 exists, Cursor agent blocked (5th attempt) |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged) | Should be closed manually |

**No new issues filed since last run.** Issues #25, #35, and #37 remain open despite having merged fixes — need manual closure by repo owner.

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Pluggable Diff Backend System | ftdv, lazygit research | P2 (new) | **ADDED** — configure external diff renderers (delta, difftastic, bat) via config |
| Diff Chunking / Smart Segmentation | Code review research (400-line threshold) | P2 (new) | **ADDED** — auto-segment large diffs at function boundaries for improved defect detection |
| Specialized Review Passes | Claude Code Review multi-agent architecture | P3 (new) | **ADDED** — guided focused review passes (logic, security, style, tests) |
| Diff Heatmap Overlay enhancement | GitTop, code review UX research | Deferred | Overlaps with existing Complexity Indicators (spec 007) — merge as visual enhancement later |
| Review Fatigue Detection | Code review 400-line research | Deferred | Enhances existing P3 Review Session Timer — not a separate item |
| Annotation Export to GitHub PR Comments | Already P2 | P1 candidate | Research continues to validate — consider promoting next cycle |

### Specs Written
No new specs this cycle — the top 3 picks already have detailed specs from previous runs:
1. `.github/specs/020-file-tree-navigator.md` (Issue #53)
2. `.github/specs/021-ask-a-question.md` (Issue #52)
3. `.github/specs/019-diff-statistics-dashboard.md`

### PRs & Agents
**Open PRs:**
- PR #54 (chore: release v0.1.19) — release automation PR, version bump only. No action needed.

**Cursor agents blocked (model unavailable — 5th consecutive run):**
- File Tree Navigator (spec 020, Issue #53)
- Diff Statistics Dashboard (spec 019)
- Ask a Question / Inline Q&A (spec 021, Issue #52)

Attempted with default model (claude-4.6-sonnet-high-thinking) and explicit model name (claude-3.5-sonnet) — both returned "Model not available or invalid." The Cursor cloud agent integration has been blocked for 5 consecutive daily runs. This is a persistent infrastructure issue requiring attention.

**Previous agent results:** All mdiff Cursor agents from earlier runs are FINISHED. No running or in-progress agents.

### Roadmap Updates
- **NEW P2**: Pluggable Diff Backend System — configure external diff renderers via config
- **NEW P2**: Diff Chunking / Smart Segmentation — auto-segment large diffs at function boundaries
- **NEW P3**: Specialized Review Passes — guided focused review passes
- Added ftdv, keifu, dead-ringer, cmux to competitive landscape
- Added "Competitive Analysis & Code Review Research" research section

### Mockups Generated
- [File Tree Navigator mockup](https://www.town.com/content/image/sc783yrt9fgzm1avw4nw51jqt183877r) — Collapsible directory hierarchy in navigator
- [Diff Statistics Dashboard mockup](https://www.town.com/content/image/sc77c8a2hfb3v1xygpd1w79kz5838z03) — Summary view with sparkline bars
- [Ask a Question mockup](https://www.town.com/content/image/sc79t8evsvqb2p54k6e8t7a9qt838nhe) — Inline Q&A with context assembly

## 2026-03-18 (Run #8)

### Research Findings
- **Anduin (Rust + Iced)**: Brand-new Git GUI specifically designed for coding-agent workflows and worktrees. Created March 16, 2026. Rust-based like mdiff but uses a GUI (Iced) rather than TUI. Directly validates mdiff's target use case — the ecosystem is converging on "tools specifically for reviewing coding agent output."
- **carn (Go TUI)**: New TUI (v0.1.0, March 17) for browsing Claude and Codex AI coding sessions. Features diff display, markdown rendering with syntax highlighting, and full-text search across session transcripts. KEY INSIGHT: Session transcript browsing alongside diffs is a pattern mdiff should adopt.
- **Ralph TUI**: Open-source terminal UI orchestrator connecting multiple AI coding agents to task trackers. Autonomous agent loops with subagent tracing and session persistence. Represents the orchestration layer pattern.
- **critique v0.1.129**: Updated to 1,091 stars (up from 1,087 in 2 days). 38 releases. Competitive pressure continues to increase.
- **Anthropic Claude Code Review multi-agent architecture**: Dispatches specialized parallel reviewers per PR, each focusing on a different failure mode. Validates mdiff's structured annotation category approach.
- **Human-in-the-loop as permanent architecture**: Industry consensus shifting from "temporary scaffold" to "genuine partnership." mdiff should optimize for permanent, not temporary, review workflows.
- **Custom instructions for AI code review (Copilot pattern)**: Natural language project-specific review criteria. Inspired Custom Review Rubric File idea.
- **78% of production sites use AI-assisted development; ~30% of dev time on code reviews**: Review efficiency is THE bottleneck.
- **GitTop (Go TUI)**: Git stats dashboard built with Bubble Tea. Activity heatmaps and contributor analytics.

### Open Issues Reviewed (5 open)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open — filed by repo owner | Spec 020 written, Cursor agent blocked (4th attempt) |
| #52 | Feature | P1 | Open — filed by repo owner | Spec 021 written, Cursor agent blocked (4th attempt) |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged) | Should be closed manually |

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Agent Session Transcript Viewer | carn, Ralph TUI | P2 (new) | **ADDED** — display agent reasoning alongside diffs |
| Custom Review Rubric File | Copilot custom instructions, CLAUDE.md ecosystem | P2 (new) | **ADDED** — project-specific .mdiff-review.md for annotation categories and checklist items |
| Review Session Persistence & Resume | carn session management | P2 (new) | **ADDED** — named review sessions with full state serialization |
| Batch File Operations (Multi-Select) | lazygit, Yazi | P3 (new) | **ADDED** — multi-select in navigator with batch operations |
| Diff Blame Integration | General dev workflow | P3 (new) | **ADDED** — git blame overlay for context lines |
| Change Impact Graph | Dependency analysis | P3 (new) | **ADDED** — dependency visualization for changed files |
| Split Diff View | Multiple competitors | REJECTED | Already exists in codebase (DiffViewMode::Split) |

### Specs Written
1. `.github/specs/022-review-progress-bar.md` — Persistent visual indicator of review completion

### PRs & Agents
**Open PRs:**
- PR #54 (chore: release v0.1.19) — release automation PR only.

**Cursor agents blocked (model unavailable — 4th consecutive run):**
- File Tree Navigator (spec 020, Issue #53)
- Diff Statistics Dashboard (spec 019)
- Review Progress Bar (spec 022)

### Mockups Generated
- [Review Progress Bar mockup](https://www.town.com/content/image/sc4dshzz7e1c0byjfc6rjg1sxd838mh7) — Block-character progress bar in navigator footer
- [Diff Statistics Dashboard mockup](https://www.town.com/content/image/sc45gwqjj83n8z4q25hnjejhbs838v5y) — Summary view with file-level stats
- [File Tree Navigator mockup](https://www.town.com/content/image/sc48szzsjvfngy1gze9hzqv5v1838rqp) — Collapsible directory hierarchy

## 2026-03-17 (Run #7)

### Research Findings
- **critique v0.1.127** (1,087 stars): Now uses tree-sitter for syntax parsing. Active competitor with 37 releases.
- **patchcast** (Rust): Novel approach converting git diffs to animated MP4 walkthroughs. Scene-based progression is an interesting UX pattern.
- **Agentic review architecture (Archbot)**: Shift from fixed LLM pipelines to agentic loops with structured "submit review" actions.
- **PR comments as agent input format**: GitHub Copilot coding agent consumes structured PR review comments. Validates mdiff's annotation export approach.
- **Agent instruction file ecosystem**: CLAUDE.md, AGENTS.md, copilot-instructions.md, GEMINI.md, CODEX.md files emerging.
- **Terminal Power Trio pattern**: Ghostty + Yazi + Lazygit as standard power-user workflow.

### Ideas Evaluated
- **PROMOTED Review Progress Bar to P1** — high impact, reasonable scope
- **PROMOTED Annotation Export to GitHub PR Comments to P2** — research confirms PR comments are standard agent input
- **NEW P2**: Inline Diff Minimap — vertical change density indicator
- **NEW P3**: Review Session Timer / Metrics
- **NEW P3**: Evidence-Based Review Annotations

### PRs & Agents
Cursor agents blocked — 2nd consecutive run with model unavailability.

## 2026-03-16 (Run #6)

### Key Actions
- **PROMOTED Issue #53 to P1**: File Tree Navigator — spec 020 written
- **PROMOTED Issue #52 to P1**: Ask a Question (Inline Q&A) — spec 021 written
- **Updated P0 statuses**: #25 FIXED, #35 FIXED, #38 CLOSED
- Cursor agents queued but model temporarily unavailable

### Research
- Duff browser-based diff viewer, gstack code review CLI, QwenLM multi-model review
- AI-native terminal design trend, GitHub Copilot feedback loop patterns, CursorBench-3

## 2026-03-13 (Run #5)

### Key Actions
- Cursor agents launched for P0 issues #25, #35, #38
- Spec 019 written (Diff Statistics Dashboard)
- NEW P3: Multi-Agent Session Comparison

### Research
- Cross-Context Review paper (decoupled review improves error detection)
- 1Code multi-agent management, Codex CLI review queuing

## 2026-03-12 (Run #4)

### Key Actions
- ADDED P0 #5: Remove `q` keybinding (Issue #38) — spec 015
- ADDED P1: Command Palette (`Ctrl+P`) — spec 018
- Cursor agent creation blocked — model unavailable

## 2026-03-07 (Run #3)

### Key Actions
- PROMOTED Issue #25 to P0
- ADDED P1 #14: Word-Level Diff, P1 #15: Approve/Reject, P1 #16: Fix cmd+K

## 2026-03-06 (Run #2)

### Key Actions
- Added P1 items (Checklist, Complexity Indicators)
- Added P2 items (Mouse, Watch Mode, Folding, Snapshot)
- Created specs 006, 007, 008

## 2026-03-05 (Run #1)

### Key Actions
- Added P1 items (Quick-Reactions, Feedback Summary, Which-Key)
- Added P2 items (Smart Review Ordering, Glob Filtering, Heatmap)
- Created specs 003, 004, 005

## 2026-03-04 (Run #0 — Initial)

### Key Actions
- Initial roadmap created
- Opened PRs for P0 items #1, #2, #3
- Competitive analysis of critique, tuicr, acre, Kaleidoscope, lazygit, fzf
