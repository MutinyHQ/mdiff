# mdiff Ideation Agent Journal

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
- **NEW P2**: Pluggable Diff Backend System — configure external diff renderers via config, inspired by ftdv and lazygit
- **NEW P2**: Diff Chunking / Smart Segmentation — auto-segment large diffs at function boundaries, research-backed
- **NEW P3**: Specialized Review Passes — guided focused review passes, inspired by Claude Code Review multi-agent architecture
- Added ftdv, keifu, dead-ringer, cmux to competitive landscape
- Added "Competitive Analysis & Code Review Research (from 2026-03-19)" research section
- Updated agent launch statuses to reflect 5th consecutive blocked attempt

### Visual Mockups Generated
- File Tree Navigator: https://www.town.com/content/image/sd7ew697s3c2rf5ehkn58637dx836e41
- Diff Statistics Dashboard: https://www.town.com/content/image/sd70k3x5scdrcgzsgq8gdmr8g5836edq
- Inline Q&A: https://www.town.com/content/image/sd7bf5hw75ptt78k82yj3sm259836p7v

### Running Agent Status
- All previous mdiff Cursor agents: FINISHED
- 3 new agents blocked by model unavailability / auth error — 5th consecutive run blocked
- **Action needed**: Cursor cloud agent integration requires investigation. The default model (claude-4.6-sonnet-high-thinking) and explicit alternatives (claude-3.5-sonnet) both fail. This has been blocking implementation for 5 daily runs (since 2026-03-16).

---

## 2026-03-18 (Run #8)

### Research Findings
- **Anduin (Rust + Iced)**: Brand-new Git GUI specifically designed for coding-agent workflows and worktrees. Created March 16, 2026. Uses Rust + Iced (GUI, not TUI). Directly validates mdiff's target use case — the ecosystem is converging on "tools specifically for reviewing coding agent output."
- **carn (Go TUI)**: New TUI (v0.1.0, March 17) for browsing Claude and Codex AI coding sessions. Features diff display with colored additions/removals, markdown rendering with syntax highlighting, and full-text search across session transcripts. KEY INSIGHT: Session transcript browsing alongside diffs is a pattern mdiff should adopt — inspired the Agent Session Transcript Viewer idea.
- **Ralph TUI**: Open-source terminal UI orchestrator connecting multiple AI coding agents to task trackers. Autonomous agent loops with subagent tracing and session persistence. Represents the orchestration layer pattern.
- **critique v0.1.129**: Updated to 1,091 stars (up from 1,087 in 2 days), 38 releases. Competitive pressure continues to increase.
- **Anthropic Claude Code Review**: Multi-agent architecture dispatching specialized parallel reviewers per PR. Validates mdiff's structured annotation category approach.
- **Human-in-the-loop as permanent architecture**: Industry consensus shifting to "genuine partnership," not temporary scaffold. mdiff should optimize for permanent review workflows.
- **Custom instructions for AI code review (Copilot pattern)**: Natural language project-specific review criteria. Inspired the Custom Review Rubric File idea.
- **78% of production sites use AI-assisted development; ~30% of dev time on code reviews**: Strongest market validation yet for mdiff's mission.
- **GitTop (Go TUI)**: Git stats dashboard with activity heatmaps built with Bubble Tea.
- **Split Diff View already exists**: Discovered that DiffViewMode::Split, render_split, and ToggleViewMode are already implemented in the codebase — removed from ideation as a candidate.

### Open Issues Reviewed (5 open)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open — filed by repo owner | Spec 020 exists, Cursor agent blocked (4th attempt) |
| #52 | Feature | P1 | Open — filed by repo owner | Spec 021 exists, Cursor agent blocked (4th attempt) |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged) | Should be closed manually |

**No new issues filed since last run.** Issues #25, #35, and #37 remain open despite having merged fixes — need manual closure by repo owner.

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Agent Session Transcript Viewer | carn, Ralph TUI | P2 (new) | **ADDED** — display agent reasoning alongside diffs |
| Custom Review Rubric File | Copilot custom instructions, CLAUDE.md ecosystem | P2 (new) | **ADDED** — project-specific review criteria |
| Review Session Persistence & Resume | carn, Ralph TUI session management | P2 (new) | **ADDED** — full state serialization for multi-session reviews |
| Batch File Operations (Multi-Select) | lazygit, Yazi | P3 (new) | **ADDED** — multi-select in navigator with batch actions |
| Diff Blame Integration | git blame, lazygit | P3 (new) | **ADDED** — human vs agent code distinction |
| Change Impact Graph | Dependency analysis tools | P3 (new) | **ADDED** — blast radius visualization (low feasibility) |
| Split Diff View (Side-by-Side) | critique, VS Code | N/A | **REJECTED** — already exists in codebase (DiffViewMode::Split) |
| Annotation Threading / Replies | GitHub PR threads | Deferred | Extends annotation system, medium priority |

### Specs Written
1. `.github/specs/022-review-progress-bar.md` — NEW: Persistent progress bar in navigator footer showing reviewed/total files with block characters, percentage, and completion celebration state

### PRs & Agents
**Open PRs:**
- PR #54 (chore: release v0.1.19) — release automation PR, version bump only. No action needed.

**Cursor agents blocked (model unavailable — 4th consecutive run):**
- File Tree Navigator (spec 020, Issue #53)
- Diff Statistics Dashboard (spec 019)
- Review Progress Bar (spec 022) — NEW this cycle

Also attempted with explicit model names (claude-3.5-sonnet) — same error. cursor_list_repos also returned auth error. The Cursor cloud agent integration appears to have a persistent authentication/model availability issue that has now blocked implementation for 4 consecutive daily runs.

**Previous agent results:** All mdiff Cursor agents from earlier runs are FINISHED. No running or in-progress agents.

### Roadmap Updates
- **NEW SPEC**: Review Progress Bar (022) — promoted from P2 to P1 last run, now spec'd and queued for implementation
- **NEW P2**: Agent Session Transcript Viewer — passive log viewing alongside diffs
- **NEW P2**: Custom Review Rubric File — project-specific review criteria via .mdiff-review.md
- **NEW P2**: Review Session Persistence & Resume — full state serialization
- **NEW P3**: Batch File Operations (Multi-Select)
- **NEW P3**: Diff Blame Integration
- **NEW P3**: Change Impact Graph
- Added Anduin, carn, Ralph TUI to competitive landscape
- Updated critique entry (v0.1.129, 1,091 stars, 38 releases)
- Added "Coding Agent Review Ecosystem & Session Browsing" research section
- Updated Open Issues Triage table with Cursor agent status

### Visual Mockups Generated
- Review Progress Bar: https://www.town.com/content/image/sd75fp3rkk9h0cvemkje3rsv5x83466m

### Running Agent Status
- All previous mdiff Cursor agents: FINISHED
- 3 new agents blocked by model unavailability / auth error — 4th consecutive run blocked
- **Action needed**: Cursor cloud agent integration may require re-authentication or the model (claude-4.6-sonnet-high-thinking) may need to be updated in the Cursor configuration

---

## 2026-03-17 (Run #7)

### Research Findings
- **critique v0.1.127** (TypeScript/Bun): Now at 1,087 stars with 37 releases. Uses tree-sitter for syntax highlighting/parsing. Key competitive features: split view, word-level diff, watch mode, glob filtering, flexible ref/branch comparison. Iterating rapidly — competitive pressure increasing.
- **patchcast** (Rust): Novel tool converting git diffs into animated MP4 video walkthroughs using syntect. Scene-based animation sequence: title card → code reveal → red-pulse deletions → green-slide additions. Interesting UX pattern for sequential file review progression.
- **Agentic review architecture (Archbot)**: Industry shift from fixed LLM pipelines to agentic loops with structured "submit review" actions. Evidence-fetching loops address context gaps that cause confidently wrong reviews. Validates mdiff's structured annotation approach.
- **PR comments as agent input format**: GitHub Copilot coding agent consumes structured PR review comments to iterate on code changes. PR comment threads are now a de facto machine-readable feedback format — directly validates mdiff's Annotation Export to GitHub PR Comments feature.
- **Agent instruction file ecosystem**: CLAUDE.md, AGENTS.md, copilot-instructions.md, GEMINI.md, CODEX.md files emerging as structured context format for AI coding agents. Review output could target these formats.
- **Terminal Power Trio pattern**: Ghostty + Yazi + Lazygit becoming the standard power-user terminal workflow. Single-key operations (lazygit's Space to stage), panel-based layouts, GPU-accelerated rendering.
- **Harpoon (neovim)**: Quick-navigation UX — bookmark and instantly jump between frequently used files. Inspired Diff Bookmarks idea.
- **difi** (Go): New TUI diff viewer with Neovim integration — another entrant in the TUI diff space.
- **No new Rust TUI diff viewers** found — mdiff's niche remains clear and underserved.

### Open Issues Reviewed (5 open)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | Open — filed by repo owner | Spec 020 written (prev run), Cursor agent blocked |
| #52 | Feature | P1 | Open — filed by repo owner | Spec 021 written (prev run), Cursor agent blocked |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged) | Should be closed manually |

**No new issues filed since last run.** Issues #25, #35, and #37 remain open despite having merged fixes — need manual closure by repo owner.

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| Review Progress Bar | Roadmap P2 | P1 (promoted) | **PROMOTED** — high impact for preventing missed files, reasonable scope |
| Inline Diff Minimap | VS Code pattern, patchcast | P2 (new) | **ADDED** — visual change density indicator, good UX improvement |
| Annotation Export to GitHub PR Comments | Roadmap P3 | P2 (promoted) | **PROMOTED** — research confirms PR comments are standard agent input format |
| Review Session Timer / Metrics | New idea | P3 (new) | **ADDED** — useful metadata for feedback export |
| Evidence-Based Review Annotations | Archbot research | P3 (new) | **ADDED** — structured finding/evidence/confidence, extends annotation system |
| Structured Export to AGENTS.md format | Agent instruction research | Deferred | Niche; existing JSON/prompt export covers most use cases |
| Animated Diff Transitions | patchcast pattern | Deferred | Visual polish, not core; TUI animation is complex |
| File Type Grouping / Smart Sort | critique, VS Code | Deferred | Overlaps with existing Smart Review Ordering (P2) |

### Specs Written
No new specs written this cycle — the top 3 picks already have specs from previous runs:
1. `.github/specs/020-file-tree-navigator.md` (Issue #53)
2. `.github/specs/021-ask-a-question.md` (Issue #52)
3. `.github/specs/019-diff-statistics-dashboard.md`

### PRs & Agents
**Open PRs:**
- PR #54 (chore: release v0.1.19) — release automation PR, version bump only. No action needed.

**Cursor agents blocked (model unavailable — 2nd consecutive run):**
- File Tree Navigator (spec 020, Issue #53) — failed to create
- Ask a Question / Inline Q&A (spec 021, Issue #52) — failed to create
- Diff Statistics Dashboard (spec 019) — failed to create

All 3 agents have been blocked for 2 consecutive runs due to Cursor model unavailability. This is becoming a persistent blocker.

**Previous agent results:** All mdiff Cursor agents from earlier runs are FINISHED. No running or in-progress agents.

### Roadmap Updates
- **Promoted Review Progress Bar** from P2 to P1 — high impact, prevents missed files
- **Promoted Annotation Export to GitHub PR Comments** from P3 to P2 — closes the feedback loop
- **Added Inline Diff Minimap** at P2 — visual orientation aid
- **Added Review Session Timer / Metrics** at P3
- **Added Evidence-Based Review Annotations** at P3
- Added patchcast to competitive landscape
- Updated critique entry with latest stats (1,087 stars, tree-sitter)
- Added "Structured Feedback & Agent Instruction Patterns" research section
- Updated agent status notes with model unavailability dates

### Visual Mockups Generated
- File Tree Navigator: https://www.town.com/content/image/sd70afgqxmaymxpsx4essaavnx83312a
- Inline Q&A: https://www.town.com/content/image/sd7b6j17tsmq00cbgt9f0kzrbs833959
- Diff Statistics Dashboard: https://www.town.com/content/image/sd7f0wfk19yper84qkqp3kwans832n6k

### Running Agent Status
- All previous mdiff Cursor agents: FINISHED
- 3 new agents blocked by model unavailability — will retry next cycle (3rd attempt)

---

## 2026-03-16 (Run #6)

### Research Findings
- **Duff** (browser-based): New multi-repo Git diff viewer launched specifically for reviewing AI coding agent output. Uses isomorphic-git + File System Access API for serverless browser-based operation. Features auto-refresh, visual image diffs, commit history graphs, multi-repo workspace persistence. Validates mdiff's target use case from a browser angle — different niche (browser vs TUI) but same pain point.
- **gstack** (Garry Tan): Open-source Claude Code toolkit with dedicated `/review` mode for production risk assessment and `/plan-eng-review` for architecture analysis. Shows growing demand for structured review workflows layered on coding agents.
- **QwenLM multi-model code review**: Feature proposal for parallel review agents across different LLMs with an arbitrator producing unified reports. Points toward multi-model review as a future pattern.
- **AI-native terminal design trend**: Awal Terminal (Rust + Swift + Metal) with auto-detecting project stacks and AI side panels. Ghostty forks adding Claude Code session summaries, activity indicators, and tab-level git diff stats. The terminal is becoming an AI-aware workspace.
- **GitHub Copilot feedback loop**: Production-scale RLHF pattern — agent generates code as PR, humans review and leave comments, agent iterates. The pull request is the structured annotation interface. Validates mdiff's annotation approach.
- **CursorBench-3**: New evaluation suite measuring coding agent quality on real developer interactions. Growing investment in measuring agent output quality.
- **No new Rust TUI diff viewers** found — mdiff's niche remains clear and underserved.

### Open Issues Reviewed (5 open)
| Issue | Category | Priority | Status | Action |
|-------|----------|----------|--------|--------|
| #53 | Feature | P1 | **NEW** — filed by repo owner | Spec 020 written, Cursor agent queued |
| #52 | Feature | P1 | **NEW** — filed by repo owner | Spec 021 written, Cursor agent queued |
| #37 | Feature | P1 | Open (PR #45 merged) | Should be closed manually |
| #35 | Bug | P0 | Open (PR #51 merged, fix verified) | Should be closed manually |
| #25 | Bug | P0 | Open (PR #50 merged, fix verified) | Should be closed manually |

**Key finding**: Issues #25, #35, and #38 all had fixes merged since last run. #38 is already closed. #25 and #35 remain open and should be closed manually. Two new high-quality feature requests (#52, #53) filed by the repo owner with detailed acceptance criteria.

### Ideas Evaluated
| Idea | Source | Priority | Verdict |
|------|--------|----------|---------|
| File Tree Navigator (#53) | Issue (owner-filed) | P1 | **SPEC WRITTEN** — Detailed tree data structures, box-drawing indent guides, directory collapse/expand, zM/zR fold commands |
| Ask a Question: Inline Q&A (#52) | Issue (owner-filed) | P1 | **SPEC WRITTEN** — AI-native differentiator, context assembly from visible hunks, answer panel, Q&A history, session persistence |
| Diff Statistics Dashboard | Roadmap (spec 019 existed) | P1 | **AGENT QUEUED** — Already spec'd, high impact for changeset triage |
| Multi-model review arbitration | QwenLM research | P3 deferred | Too complex for this cycle, requires agent infrastructure changes |
| Auto-refresh / watch mode | Duff competitive feature | P2 existing | Already on roadmap, no urgency |
| AI session side panel | Awal Terminal pattern | P3 deferred | Interesting UX but requires fundamental layout changes |
| Review workflow templates (gstack-style) | gstack research | P2 deferred | Partially covered by existing checklist templates |

### Specs Written
1. `.github/specs/020-file-tree-navigator.md` — Collapsible directory hierarchy for navigator, TreeNode/TreeNodeKind types, dual render path, box-drawing indent guides, zM/zR fold commands, search integration
2. `.github/specs/021-ask-a-question.md` — Inline Q&A over diff context, Ctrl+Q trigger, QAState with history, context assembly from hunks/selection, answer panel as right-side split, session persistence

### PRs & Agents
**No open PRs** — all PRs from previous runs have been merged. Repository is at v0.1.18 (released 2026-03-14).

**Cursor agents queued** (model temporarily unavailable):
- File Tree Navigator (spec 020, Issue #53)
- Ask a Question / Inline Q&A (spec 021, Issue #52)
- Diff Statistics Dashboard (spec 019)

**Previous agent results confirmed**:
- PR #50 (Diff view scroll bounds, Issue #25) — merged 2026-03-14
- PR #51 (Global search keybinding fix, Issue #35) — merged 2026-03-13

### Roadmap Updates
- **Promoted #53** (File Tree Navigator) to P1 with spec 020
- **Promoted #52** (Ask a Question / Inline Q&A) to P1 with spec 021
- **Updated P0 statuses**: #25 FIXED (PR #50), #35 FIXED (PR #51), #38 CLOSED
- Added Duff, gstack to competitive landscape
- Added "AI-Native Terminal & Inline Q&A Patterns" research section
- Updated Open Issues Triage table with current status of all 5 open issues

### Visual Mockups Generated
- File Tree Navigator: https://www.town.com/content/image/sd787p99a66vc80hzqz532reyn831fav
- Ask a Question Inline Q&A: https://www.town.com/content/image/sd74mgff0bve15e398qzvnebq58304nv
- Diff Statistics Dashboard: https://www.town.com/content/image/sd73ywx841ppey573fcktq6r518314xj

### Running Agent Status
- All previous mdiff Cursor agents: FINISHED
- 3 new agents queued but blocked by model unavailability — will retry next cycle

---

## 2026-03-05 (Run #2)

### Research Findings
- **Deff** (Rust TUI) just launched on Hacker News with 37 points, 19 comments. Very early stage (8 commits, 1 star). Has side-by-side view, vim motions, per-file review toggles. No annotation or agent feedback features — validates the space but not a competitive threat.
- **HN commenter explicitly requesting** a TUI diff tool with inline commenting for coding agent feedback — directly validates mdiff's core value proposition.
- **RLHF research evolution**: DPO, GRPO, and RLVR paradigms reducing reliance on traditional reward models. Active learning principles (selecting most informative data points for human labeling) directly applicable to review prioritization.
- **AI code review insight**: High false-positive rates train developers to ignore tools entirely — key failure mode to design against. Informed the Diff Complexity Indicators feature.
- **Bottleneck shift**: Code generation is no longer the bottleneck; code review, design understanding, and safe deployment are. Human judgment irreplaceable for intent and architecture.
- **Ratatui ecosystem**: 18,790+ stars, thriving. tachyonfx for animations, ratzilla for WebAssembly TUIs.
- **critique** (TypeScript/Bun) latest release Feb 27 with watch mode, glob filtering — competitive features to track.

### Ideas Evaluated
| Idea | Priority | Verdict |
|------|----------|---------|
| Review Checklist Templates | P1 | **SPEC WRITTEN** — Novel structured feedback mechanism, high differentiation |
| Diff Complexity Indicators | P1 | **SPEC WRITTEN** — Active learning-informed triage, feasible with heuristics |
| Mouse Support for Navigation | P2 | **SPEC WRITTEN** — Low-hanging fruit, already capturing events |
| Active Learning Priority Queue | Deferred | Subsumes/enhances #16 Smart Review Ordering, too complex for this cycle |
| Diff Snapshot / Time Travel | P2 added | Agent iteration comparison, complex but differentiated |
| Watch Mode with Auto-Refresh | P2 added | Competitive feature from critique |
| Syntax-Aware Folding | P2 added | Would benefit from tree-sitter, higher complexity |
| Review Session Comparison | Deferred | Partially covered by #9 scope |
| Annotation Templates | Deferred | Builds on structured feedback but lower priority |
| Clipboard-Aware Paste | Deferred | Too small scope for this cycle |

### Specs Written
1. `.github/specs/006-review-checklist.md` — Configurable per-project review checklists with panel UI, session persistence, prompt output integration
2. `.github/specs/007-complexity-indicators.md` — Heuristic-based complexity scoring per hunk/file with colored badges in navigator and diff view
3. `.github/specs/008-mouse-support.md` — Scroll wheel, click-to-select, click-to-focus with MouseContext hit-testing

### PRs & Agents
**New Cursor Agents Launched:**
- `bc-0f90090b` — Review Checklist Templates (spec 006) → targeting develop
- `bc-991de31d` — Diff Complexity Indicators (spec 007) → targeting develop
- `bc-95322710` — Mouse Navigation Support (spec 008) → targeting develop

**Existing Open PRs Reviewed:**
- PR #12 (Annotation Categories): Found session persistence issue — loaded annotations hardcode category=Suggestion, severity=Minor. Needs AnnotationEntry format update.
- PR #13 (Global Search): Clean implementation. Minor concern about search bar z-ordering overlap with category picker.
- PR #14 (Line Scores): Good implementation. Number keys 1-5 in both visual and normal diff mode by design. Gutter alignment concern with score indicators.
- PR #15 (Feedback Summary): Uses placeholder score_count()/all_scores_sorted() — has dependency on PR #14. LineScore type will conflict. Must merge #14 before #15.
- PR #18 (Release v0.1.12): Release PR, left alone.

**Cross-PR Issues:**
- All 4 feature PRs target `main` instead of `develop` (created before branch strategy)
- PR #14 and #15 have merge conflict risk in annotation_state.rs (competing LineScore definitions)
- PR #12 and #14 both modify event.rs key mappings

### Roadmap Updates
- Updated ROADMAP.md with new P1 items (#11 Review Checklist, #12 Complexity Indicators)
- Added P2 items (#21 Mouse Support, #22 Watch Mode, #23 Syntax-Aware Folding, #24 Diff Snapshot)
- Updated statuses: #1 Hunk Navigation → Merged, #10 Which-Key → Merged
- Added Deff to competitive landscape
- Added HN Signal section and RLHF/Active Learning Insights section

### Visual Mockups Generated
- Review Checklist Panel: https://www.town.com/content/image/sd70y16gvj3f605wwbdy3zb9m582a9rh
- Diff Complexity Indicators: https://www.town.com/content/image/sd70n8zdpcgag8cr0wpvyy6n0d82b7vf
- Mouse Navigation Support: https://www.town.com/content/image/sd7c982v5ttgsn5pe72kzwteq582aday

### Running Agent Status
- `bc-99b3926c` — "Development environment setup" on mdiff (RUNNING, started by someone else, not from ideation workflow)
- `bc-0f90090b` — Review Checklist Templates (CREATING)
- `bc-991de31d` — Diff Complexity Indicators (CREATING)
- `bc-95322710` — Mouse Navigation Support (CREATING)
