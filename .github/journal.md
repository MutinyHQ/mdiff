# mdiff Ideation Agent Journal

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
