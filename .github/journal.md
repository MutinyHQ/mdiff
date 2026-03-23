# mdiff Ideation Agent Journal

## 2026-03-23 (Run #12)

### Research Findings
- **TUI Resurgence** (Leon Mika blog, 2026-03-21): TUIs experiencing resurgence driven by AI coding agents (Claude Code, Codex). Key advantages: SSH access, terminal context, cheaper than GUIs. Strongest validation of mdiff's terminal-native approach.
- **Muse/Manyana conflict-aware diffs** (GitHub issue #237): Novel diff UX labeling each hunk with the action and which side performed it. Directly inspired Agent Attribution Labels (spec 026).
- **oh-my-pi** (AI terminal agent): Agentic git tools for fine-grained diff analysis. Automatic atomic commit splitting. Inspired Commit Split Suggestions idea.
- **Codex diff rendering bug (#15416)**: Green-on-green illegible diffs on terminals without true color support. Inspired Terminal Compatibility feature.
- **Claude Code TUI flickering (#37283)**: Non-atomic screen redraws cause flickering. DECSET 2026 synchronized output is the fix. Inspired Synchronized Output feature.
- **Agent-RLVR (Scale Labs)**: Structured "agent guidance" improves coding agent performance from 9.4% to 22.4% on SWE-Bench. Directly inspired Agent Guidance Export (spec 027).
- **Superset**: Desktop workspace for multi-agent orchestration with built-in diff review. Competing GUI approach.
- **Orchestrator**: tmux-style split panes for parallel Claude Code sessions.
- **Gitea inline commit diff comments** (PR #36944): Commit-level code review workflow pattern.

### Critical Fix: ROADMAP.md Corruption (Again)
ROADMAP.md on main corrupted again — contained literal sandbox path from Run #11. The `github_create_or_update_file` tool treats file paths passed as `content` parameter as literal text. Launched Cursor agent `bc-5b038023` to restore from commit `aad15b4` and apply 2026-03-23 updates. For large files (>25K chars), Cursor agents are the only reliable commit method going forward.

### Open Issues Reviewed (5 open — unchanged)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | PR #56 merged | **Implemented**. Issue should be closed. |
| #52 | Feature | P1 | Open | Spec 021 exists. Needs re-launch. |
| #37 | Feature | P1 | PR #45 merged | Should be closed manually |
| #35 | Bug | P0 | PR #51 merged | Should be closed manually |
| #25 | Bug | P0 | PR #50 merged | Should be closed manually |

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Agent Attribution Labels | Muse/Manyana | P1 (new) | **ADDED** — spec 026 |
| Agent Guidance Export | Agent-RLVR, CLAUDE.md | P1 (new) | **ADDED** — spec 027 |
| Review Session Bookmarks | Vim marks, VS Code | P2 (new) | **ADDED** — spec 028 |
| Commit Split Suggestions | oh-my-pi | P2 (new) | **ADDED** to roadmap |
| Terminal Compatibility | Codex bug #15416 | P2 (new) | **ADDED** to roadmap |
| Synchronized Output | Claude Code bug #37283 | P2 (new) | **ADDED** to roadmap |
| Diff Rendering Quality Indicators | Code review research | P3 (new) | **ADDED** to roadmap |
| Commit-Level Annotation Export | Gitea | P3 (new) | **ADDED** to roadmap |

### Specs Written
1. `.github/specs/026-agent-attribution-labels.md` — Hunk-level agent session markers with gutter markers, session legend, filter-by-session.
2. `.github/specs/027-agent-guidance-export.md` — Three new export formats: Agent Instruction prompt, CLAUDE.md patch, Git trailers.
3. `.github/specs/028-review-session-bookmarks.md` — Quick line markers with single-key toggle, named bookmarks, navigation, gutter markers.

### PRs Reviewed
- **PR #55** (Annotation Code Suggestions) — +911/-66, 14 files. Well-structured, follows patterns. No issues.
- **PR #57** (Open-in-Editor at Line) — +197/-0, 5 files. Clean implementation. No issues.
- **PR #56** (File Tree Navigator) — Merged. +1213/-195, 6 files.
- **PR #54** (release v0.1.19) — Version bump only.

### Cursor Agents
**Completed from Run #11:** bc-85ef5215 (PR #55), bc-34758cf7 (PR #57), bc-f6fc2377 (PR #56 merged)
**Launched in Run #12:** bc-cc08ca00 (spec 026), bc-89603837 (spec 027), bc-7b538030 (spec 028), bc-5b038023 (ROADMAP fix)

### Mockups Generated
- [Agent Attribution Labels](https://www.town.com/content/image/sd751fkvc5gbqpkyw1cc367fjd83e47a)
- [Agent Guidance Export](https://www.town.com/content/image/sd7bs8npb4actyzks4tj943fj983e3zf)
- [Review Session Bookmarks](https://www.town.com/content/image/sd79ezg99z09exhk3bn7ct4q0983f276)

## 2026-03-20 (Run #11)

### Research Findings
- **critique v0.1.129** (1,091 stars): Responsive split view pattern — auto-switches to unified on narrow terminals. Validates new P2 Responsive Layout Mode idea.
- **Horizon (Rust, 221 stars)**: GPU-accelerated terminal board with infinite 2D canvas, minimap, workspaces, and first-class AI agent integration (dedicated Claude Code/Codex panels). Trending March 2026. Minimap pattern validates existing P2 Inline Diff Minimap idea.
- **NTM (Go)**: Named Tmux Manager for orchestrating multiple coding agents with conflict detection. Validates multi-agent review workflow.
- **HubSpot Sidekick**: AI code review with 80% engineer approval rate and 90% faster time-to-first-feedback. Human approve/reject of AI review comments is an emerging implicit RLHF signal — directly validates mdiff's annotation system as structured feedback data source.
- **CodeRabbit (2M+ repos)**: Line-by-line review with 1-click commit to apply fixes. Accept/reject/modify interaction loop generates implicit human preference data. Directly inspired the Quick Apply Suggestion spec (025).
- **Robin Wieruch's agentic review pattern**: Documenting project patterns/anti-patterns in structured files, then AI reviews against them. Validates existing P2 Custom Review Rubric File idea.
- **fzf v0.70.0**: 1.3-1.9x filtering performance improvements. Perceived speed is critical UX factor — inspired P2 Lazy Virtual Scrolling idea.
- **lazygit custom commands**: YAML-configured prompts with templated commands. Extensibility pattern for TUI tools.
- **GitHub March 2026 agentic workflows**: Configurable validation tools and complete audit logs for AI code review. Infrastructure for structured feedback at scale.
- **"Visual confirmation before commit" pattern**: Developers explicitly building 4-pane terminal layouts (Claude Code + dev server + browser + lazygit) to review agent output before committing. Strongest ongoing validation of mdiff's core purpose.
- **Code review bottleneck**: AI agents produce 200%+ more code per engineer, creating urgent demand for scalable review feedback mechanisms.

### Critical Fix: ROADMAP.md Corruption
Discovered that the ROADMAP.md file on `main` had been accidentally overwritten with a sandbox path reference (`sandbox:///home/user/session/roadmap_updated.md`) during Run #10's commit. The file went from 449 lines to 1 line in commit `ae3e7f4`. Recovered the full content from the commit diff, applied all updates from this run, and restored the file in commit `9aa607c`.

### Open Issues Reviewed (5 open — unchanged)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open — filed by repo owner | Spec 020 exists, Cursor agent LAUNCHED (run #11) |
| #52 | Feature | P1 | Open — filed by repo owner | Spec 021 exists, not in this run's top 3 |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged) | Should be closed manually |

**No new issues filed since last run.** Issues #25, #35, and #37 continue to need manual closure by repo owner — fixes have been merged for 8+ days now.

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Quick Apply Suggestion (Ctrl+A) | CodeRabbit 1-click apply, GitHub commit suggestion | P1 (new) | **ADDED** — spec 025, write suggestion to working tree file |
| Responsive Layout Mode | critique responsive split view | P2 (new) | **ADDED** — auto-switch split/unified based on terminal width |
| Lazy Virtual Scrolling for Large Diffs | fzf performance, Horizon | P2 (new) | **ADDED** — viewport-based rendering for 1000+ line diffs |
| Review Confidence Self-Assessment | Anthropic confidence scoring, HubSpot Sidekick | P3 (new) | **ADDED** — per-file confidence score (1-5) in feedback export |
| Agent Instruction Export (.claude.md patches) | CLAUDE.md ecosystem, Robin Wieruch | Deferred | Interesting but overlaps with existing Annotation Export to GH PR Comments — revisit later |
| Configurable Custom Commands (YAML) | lazygit custom commands | Deferred | Nice extensibility but low priority vs core features |
| Audit Log of Review Actions | GitHub agentic workflow audit logs | Deferred | Useful for compliance but not core reviewer experience |

### Specs Written
One new spec this cycle:
1. `.github/specs/025-quick-apply-suggestion.md` — One-key apply of annotation suggestions to the working tree file. Ctrl+A trigger, confirmation dialog, auto-refresh, undo via Ctrl+Z. Depends on Annotation Suggestions (spec 023).

### PRs & Agents
**Open PRs:**
- PR #54 (chore: release v0.1.19) — release automation PR, version bump only. Reviewed; no action needed. Now includes commits from this run (spec 025, ROADMAP restore).

**Cursor agents LAUNCHED SUCCESSFULLY (first successful launch since Run #5 on 2026-03-13):**
- `bc-f6fc2377` — File Tree Navigator (spec 020, Issue #53) — Status: CREATING, branch: `cursor/file-tree-navigator-1977`
- `bc-85ef5215` — Annotation Suggestions (spec 023) — Status: RUNNING, branch: `cursor/annotation-code-suggestions-5251`
- `bc-34758cf7` — Open-in-Editor (spec 024) — Status: CREATING, branch: `cursor/external-editor-support-4507`

This is a major breakthrough — Cursor agents had been blocked for 8 consecutive runs (runs #3 through #10) due to model unavailability. All 3 agents were created without model specification errors this run.

**Previous agent results:** All mdiff Cursor agents from earlier runs are FINISHED. No stale agents.

### Mockups Generated
- [File Tree Navigator mockup](https://www.town.com/content/image/sd76dnfmbnns8w8ap2dv0fjd1x838e6c) — Collapsible directory tree with box-drawing indent guides and aggregate stats
- [Annotation Suggestions mockup](https://www.town.com/content/image/sd73kzhg1nmmj3csw6k6yc0gnn838k2r) — Two-pane code suggestion editor overlay
- [Quick Apply Suggestion mockup](https://www.town.com/content/image/sd7544zk6fxth9x01qq1rwe2hd839szr) — Confirmation dialog with apply/cancel and applied indicator

### Roadmap Updates
- **CRITICAL FIX**: Restored ROADMAP.md from corruption (commit `9aa607c`)
- **NEW P1**: Quick Apply Suggestion (`Ctrl+A`) — spec 025
- **NEW P2**: Responsive Layout Mode — auto-switch split/unified based on terminal width
- **NEW P2**: Lazy Virtual Scrolling for Large Diffs — viewport-based rendering
- **NEW P3**: Review Confidence Self-Assessment — per-file confidence score (1-5)
- Added Plannotator, Horizon, NTM to competitive landscape
- Updated changelog with 2026-03-20 entry

### Key Insight This Cycle
The Cursor agent infrastructure blockage has finally been resolved — after 8 consecutive runs of model unavailability, all 3 agents launched successfully this run. One agent ("Annotation code suggestions") reached RUNNING status within seconds. This means the implementation backlog that has been accumulating (specs 019-025) can finally start being worked through. The top 3 picks for this cycle (File Tree Navigator, Annotation Suggestions, Open-in-Editor) are all high-impact P1 features that have been waiting for implementation.

The Quick Apply Suggestion spec (025) completes a three-feature pipeline: Annotation Suggestions (023) lets reviewers propose code -> Quick Apply (025) writes it to disk -> the diff auto-refreshes. This review-suggest-apply-refresh loop is the most complete structured feedback cycle in any TUI diff viewer.

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
- Researched RLHF/human feedback patterns for annotation design
