# mdiff Ideation Agent Journal

## 2026-03-07 (Run #3)

### Research Findings
- **Deff** continues gaining traction on HN (37+ points, 19 comments). Commenters explicitly requesting inline annotation for agent feedback — directly validates mdiff's unique positioning. Deff has no annotation features, only per-file "reviewed" toggles.
- **review-for-agent** (Waraq-Labs): New tool providing local GitHub-style diff view with inline comments and structured JSON/Markdown export. Validates our spec 011 (structured feedback export) — their approach is browser-based, ours is terminal-native.
- **git-lanes** (12 stars, TypeScript): Parallel AI agent isolation tool with dedicated worktrees per agent. Shows the multi-agent workflow ecosystem is expanding. Adjacent to mdiff's worktree management.
- **claude-compaction-viewer** (swyx, Python TUI): Inspects Claude Code conversation history and compaction events. Growing niche of AI-agent-specific TUI developer tools.
- **justshowmediff** (Go): Generates self-contained HTML diff files for browser viewing. Targets Claude Code/Codex post-tool hooks. Zero-dependency approach, different philosophy from TUI.
- **diffreview** (Zsh helpers): Pipes git diffs into Claude Code or Copilot CLI for LLM-powered review. Shows the trend of integrating diff tools with LLM agents.
- **RLHF research**: Comparative/ranking interfaces more effective than freeform evaluation. Humans compare outputs more easily than they evaluate individual outputs. Informs new P3 idea: Side-by-Side Agent Comparison View.
- **REEL pattern** (Review Evaluated Engineering Loop): Formalizes spec-in, fresh sessions per task, acceptance gates. mdiff's verdict system maps directly to the acceptance gate step.
- **AI code review failure modes**: High false-positive rates cause reviewer disengagement. AI-generated code needs fundamentally different review workflows (no implicit context, high confidence without correctness, volume overwhelming attention).
- **Lazygit UX research**: Issue #1712 discusses balancing discoverability vs power-user efficiency in keyboard-driven TUIs. Key tension directly applicable to mdiff's growing keybinding surface area.
- **Layered review architecture**: Automated first-pass (lint, types, naming) + human second-pass (architecture, design, business logic). mdiff operates at the human layer — should not try to automate what belongs in the first pass.

### Open Issues Triage
| Issue | Type | Priority | Verdict |
|-------|------|----------|---------|
| #27: Which-key dialog flashes | Bug | P0 | Spec ready, Cursor agent launched |
| #25: Diff line calculations off | Bug | P0 | RESOLVED — PR #26 merged |
| #24: cmd+K kills wrong session | Bug | P1 | Spec ready, Cursor agent launched |

### Ideas Evaluated
| Idea | Priority | Verdict |
|------|----------|---------|
| Fix Which-Key Flash (#27) | P0 | **CURSOR AGENT LAUNCHED** — Small fix, high impact on discoverability |
| Fix cmd+K Kill Wrong Session (#24) | P1 | **CURSOR AGENT LAUNCHED** — Index mapping bug fix |
| Structured Feedback Export (spec 011) | P1 | **CURSOR AGENT LAUNCHED** — Critical for agent feedback loop, validated by review-for-agent |
| Approve/Reject Agent Workflow (spec 011) | P1 | Deferred — needs export to land first, good next-cycle pick |
| Bookmark/Pin Lines for Quick Return | P2 | **ADDED TO ROADMAP** — Vim marks for large diff review |
| Review Progress Bar / Session Timer | P2 | **ADDED TO ROADMAP** — Awareness of review pacing |
| Annotation Quick-Reply Templates | P2 | **ADDED TO ROADMAP** — Pre-configured feedback snippets |
| Export to GitHub PR Comment | P2 | **ADDED TO ROADMAP** — Close the TUI-to-PR feedback loop |
| Side-by-Side Agent Comparison View | P3 | **ADDED TO ROADMAP** — RLHF comparative interface, novel capability |

### Specs Referenced (Not New This Cycle)
- `.github/specs/010-which-key-flash-fix.md` — Already existed from previous commit
- `.github/specs/011-structured-feedback-export.md` — Already existed from previous commit
- `.github/specs/012-agent-session-kill-fix.md` — Already existed from previous commit

### PRs & Agents
**New Cursor Agents Launched:**
- `bc-6a60531f` — Fix Which-Key Dialog Flashing (spec 010, Issue #27) → targeting develop
- `bc-90f4ab2d` — Fix cmd+K Kill Wrong Session (spec 012, Issue #24) → targeting develop
- `bc-a6cb920d` — Structured Feedback Export (spec 011) → targeting develop

**Previous Cycle Agent Results:**
- `bc-c2718d68` — Review Decision Workflow: **ERRORED** (model issue, was targeting develop)
- `bc-63ab099f` — Word-Level Diff Highlighting: **FINISHED** → PR #28 **MERGED** to develop
- `bc-81795ecd` — Diff Line Calculations: **FINISHED** → PR #26 **MERGED**
- `bc-991de31d` — Diff Complexity Indicators: **FINISHED** → PR #22 **MERGED** to develop
- `bc-95322710` — Mouse Support: **EXPIRED** → PR #20 closed, but PR #30 manually created and **MERGED** to main
- `bc-0f90090b` — Review Checklist: **FINISHED** → PR #21 closed (targeted main, not develop)

**Existing Open PRs Reviewed:**
- PR #12 (Annotation Categories): STALE draft, targets main. 3+ days old with no updates. Implementation adds category picker and severity levels but has known session persistence issue (hardcodes category=Suggestion). Needs retarget to develop or close.
- PR #14 (Annotation Line Scores): STALE draft, targets main. 3+ days old. Adds 1-5 scoring with number keys, gutter indicators. Needs retarget to develop or close.

**Cross-PR Observations:**
- PRs #12 and #14 were created before the develop branch strategy was established. Both target main. They should either be retargeted to develop or closed and re-implemented on develop.
- The review-decision-workflow agent (bc-c2718d68) errored due to a model availability issue. The approve/reject workflow (spec 011-approve-reject-workflow) remains unimplemented.

### Roadmap Updates
- Updated ROADMAP.md with 2026-03-08 cycle changes
- Promoted Issue #27 to P0 as item #5
- Updated Issue #25 status to RESOLVED
- Updated merge statuses: Complexity Indicators, Word-Level Diff, Mouse Support all merged
- Added 4 new P2 ideas: Bookmark/Pin Lines (#22), Review Progress Bar (#23), Annotation Quick-Reply Templates (#24), Export to GitHub PR Comment (#25)
- Added 1 new P3 idea: Side-by-Side Agent Comparison View (#26)
- Expanded competitive landscape with git-lanes, review-for-agent, claude-compaction-viewer, difit
- Added "AI Code Review Failure Modes" research section
- Updated research notes with RLHF comparative interface insights, REEL pattern, layered review architecture

### Visual Mockups Generated
- Which-Key Help Overlay Fix: https://www.town.com/content/image/sd73fy0xfvxft6mkvebqrf5hts82fbt8
- Structured Feedback Export: https://www.town.com/content/image/sd751cmc571ka9pbhxt6v632mh82efge
- Agent Session Kill Fix: https://www.town.com/content/image/sd79h0z5hs1hyc9d8rnj0rekcx82e2mz

### Running Agent Status
- `bc-6a60531f` — Fix Which-Key Dialog Flashing (RUNNING)
- `bc-90f4ab2d` — Agent Session Index Mapping / Kill Fix (RUNNING)
- `bc-a6cb920d` — Structured Feedback Export (CREATING)

### Key Decisions & Rationale
1. **Prioritized bug fixes over new features**: Issues #27 and #24 are user-reported bugs that directly impact usability. Bug fixes take precedence over new feature development.
2. **Selected structured export as the feature pick**: Validated by competitive intelligence (review-for-agent), research (REEL pattern "state on disk"), and the core mission (machine-readable feedback for agent consumption). This is the most impactful unimplemented spec.
3. **Deferred approve/reject workflow**: While the spec is ready and validated by research, it has a soft dependency on the export system — the verdict should be included in the export schema. Better to land export first, then add verdict integration.
4. **New P2/P3 ideas added but not implemented**: Bookmark/Pin Lines, Review Progress, Templates, GitHub Export, and Agent Comparison are all validated by research but lower priority than fixing bugs and landing the export system.

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
