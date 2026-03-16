# mdiff Ideation Agent Journal

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
