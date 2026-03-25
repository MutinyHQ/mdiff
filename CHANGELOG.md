# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.21](https://github.com/MutinyHQ/mdiff/compare/v0.1.20...v0.1.21) - 2026-03-25

### Fixed

- agent output focus trap

## [0.1.20](https://github.com/MutinyHQ/mdiff/compare/v0.1.19...v0.1.20) - 2026-03-24

### Added

- agentic review

### Fixed

- remove stale agentic_review_running field from test KeyContext
- which key modal
- resolve 5 tree view inconsistencies (issue #64)

### Other

- Merge branch 'main' into cursor/tree-view-consistency-8764
- add 031-goto-line-and-file-picker for Issue #63
- add 030-file-preview-on-hover for Issue #65
- add 029-tree-view-inconsistencies for Issue #64

## [0.1.19](https://github.com/MutinyHQ/mdiff/compare/v0.1.18...v0.1.19) - 2026-03-23

### Added

- add file tree navigator with collapsible directory hierarchy

### Other

- Merge pull request #60 from MutinyHQ/cursor/review-session-bookmarks-d848
- add Run #12 entry (2026-03-23) — specs 026-028, Cursor agents launched, ROADMAP fix, PR reviews
- restore from corruption + add P1 Agent Attribution Labels/Agent Guidance Export, P2 Bookmarks/Commit Split/Terminal Compat/Sync Output, P3 Quality Indicators/Git Notes, 2026-03-23 changelog
- add 028-review-session-bookmarks for quick line markers during review
- add 027-agent-guidance-export for structured feedback as agent instructions
- add 026-agent-attribution-labels for hunk-level agent session markers
- restore from corruption + add P1 Quick Apply Suggestion, P2 Responsive Layout/Lazy Scrolling, P3 Review Confidence, new competitors (Plannotator, Horizon, NTM), 2026-03-20 changelog
- add 025 quick apply suggestion (write suggestion to working tree)
- add run #10 entry — Plannotator discovery, 2 new specs (023, 024), Cursor agents blocked 7th run
- add P1 Annotation Suggestions + Open-in-Editor, new competitors, 2026-03-20 changelog
- add 024 open-in-editor at line
- add 023 annotation suggestions (inline code proposals)
- add 2026-03-19 entry — ftdv competitive discovery, 3 new roadmap ideas (Pluggable Backends, Diff Chunking, Review Passes), Cursor agents blocked 5th run, mockups generated
- update for 2026-03-19 — new P2 items (Pluggable Diff Backend, Diff Chunking), new P3 (Specialized Review Passes), ftdv/keifu/cmux/dead-ringer competitive entries, code review research (400-line threshold)
- add 2026-03-18 entry — spec 022 (Review Progress Bar), 6 new roadmap ideas, Anduin/carn/Ralph TUI research, Cursor agents blocked 4th run
- update for 2026-03-18 — spec 022 (Review Progress Bar), new P2/P3 ideas (Agent Transcript Viewer, Custom Review Rubric, Session Persistence, Batch Ops, Blame, Impact Graph), Anduin/carn/Ralph TUI competitive entries, research on agent review ecosystem
- add 022 - Review Progress Bar with file-level completion tracking
- add 2026-03-17 entry — research on critique/patchcast/agentic review, promoted Review Progress Bar to P1, new P2/P3 ideas, agents blocked by model unavailability (2nd run)
- update for 2026-03-17 — promote Review Progress Bar to P1, add Inline Diff Minimap (P2), promote GitHub PR Comment Export (P2), new P3 ideas, research on critique/patchcast/agentic review patterns
- add 2026-03-16 entry — new issues #52/#53 triaged, specs 020/021 written, research on Duff/gstack/AI-native terminals, mockups generated
- update for 2026-03-16 — promote Issues #52 and #53 to P1, update P0 statuses, add Duff/gstack to competitive landscape, new research section
- add 021 - Ask a Question inline Q&A over diff context (Issue #52)
- add 020 - File Tree Navigator with collapsible directory hierarchy (Issue #53)

## [0.1.18](https://github.com/MutinyHQ/mdiff/compare/v0.1.17...v0.1.18) - 2026-03-14

### Other

- Fix ScrollToBottom action to show actual bottom of diff
- add 019 - Diff Statistics Dashboard (S toggle) for changeset overview

## [0.1.17](https://github.com/MutinyHQ/mdiff/compare/v0.1.16...v0.1.17) - 2026-03-12

### Other

- update for 2026-03-12 — add Command Palette (P1), promote Issue #38 to P0, add research notes, triage open issues
- add 018 - Command Palette (Ctrl+P) for action discoverability

## [0.1.16](https://github.com/MutinyHQ/mdiff/compare/v0.1.15...v0.1.16) - 2026-03-12

### Other

- Merge pull request #47 from MutinyHQ/cursor/viewport-review-marking-1bcb
- Fix agent session kill index mapping
- add 016 - Viewport-Based File Review Marking (Issue #43)
- add 017 - Add New Opencode Review Models (Issue #37)

## [0.1.15](https://github.com/MutinyHQ/mdiff/compare/v0.1.14...v0.1.15) - 2026-03-10

### Other

- Merge pull request #42 from MutinyHQ/cursor/paste-event-handling-2c17
- Fix paste bug in comment modal and other text inputs
- add 015 - Remove q Keybinding for Quit (Issue #38)
- add 014 - Fix Paste in Comment Modal (Issue #36)
- add 013 - Fix Global Search UX (Issue #35)

## [0.1.14](https://github.com/MutinyHQ/mdiff/compare/v0.1.13...v0.1.14) - 2026-03-09

### Added

- implement annotation quick-reactions (line scoring)
- Add annotation categories and severity levels

### Other

- Merge pull request #14 from MutinyHQ/cursor/annotation-line-scores-e94d
- Fix which-key dialog flashing bug (Issue #27)
- Merge branch 'develop'

## [0.1.13](https://github.com/MutinyHQ/mdiff/compare/v0.1.12...v0.1.13) - 2026-03-06

### Added

- implement configurable review checklist system
- Add mouse support for navigation

### Other

- cargo fmt
- Fix diff line calculations for ScrollToBottom action

## [0.1.12](https://github.com/MutinyHQ/mdiff/compare/v0.1.11...v0.1.12) - 2026-03-06

### Added

- implement Agent Feedback Summary View
- ellipsis diff paths in the middle

### Fixed

- address clippy warnings and format code

### Other

- Add global fuzzy search across all diff content (Ctrl+F)
- add workflow_dispatch trigger for manual runs
- Add AGENTS.md with Cursor Cloud specific instructions
- Merge branch 'main' into cursor/which-key-overlay-23bc
- Merge pull request #19 from MutinyHQ/alechoey/confirm-exit

## [0.1.11](https://github.com/MutinyHQ/mdiff/compare/v0.1.10...v0.1.11) - 2026-03-04

### Fixed

- clippy
- PTY process cleanup, kill targeting, and paste support

### Other

- add implementation spec for annotation quick-reactions
- add implementation spec for contextual help overlay (which-key)
- add implementation spec for agent feedback summary view
- add 7 new feature ideas from competitive research (2026-03-05)
- add implementation spec for annotation categories and severity levels
- add implementation spec for hunk-level navigation
- add comprehensive roadmap and feature backlog

## [0.1.10](https://github.com/MutinyHQ/mdiff/compare/v0.1.9...v0.1.10) - 2026-02-20

### Added

- 3-column split layout with shared center gutter
- side-aware annotation anchors and shell text navigation

### Fixed

- suppress clippy too_many_arguments warning
- align diff scrolling with visual rows

### Other

- extract WrapConfig struct from wrap function args

## [0.1.9](https://github.com/MutinyHQ/mdiff/compare/v0.1.8...v0.1.9) - 2026-02-19

### Fixed

- truncate file paths from the left in navigator on narrow screens
- synchronize line wrapping in split diff view

## [0.1.8](https://github.com/MutinyHQ/mdiff/compare/v0.1.7...v0.1.8) - 2026-02-18

### Added

- add worktree indicator and switch action to agent outputs

### Fixed

- use accurate diff viewport height from render layout

## [0.1.7](https://github.com/MutinyHQ/mdiff/compare/v0.1.6...v0.1.7) - 2026-02-18

### Added

- use 3-dot diff (merge-base) for branch and commit comparisons

## [0.1.6](https://github.com/MutinyHQ/mdiff/compare/v0.1.5...v0.1.6) - 2026-02-18

### Added

- forward mouse scroll to PTY in agent outputs view

### Fixed

- use content highlight instead of gutter color for search matches
- set agent subprocess cwd to repo_path so agents start in the correct directory

## [0.1.5](https://github.com/MutinyHQ/mdiff/compare/v0.1.4...v0.1.5) - 2026-02-18

### Added

- fix file search selection and add diff text search

### Fixed

- render agent output from bottom so cursor line is always visible

## [0.1.4](https://github.com/MutinyHQ/mdiff/compare/v0.1.3...v0.1.4) - 2026-02-18

### Added

- auto-review on scroll-to-bottom, add review keys to HUD
- persist last-used model per agent CLI, PTY runner, review tracking
- allow V to toggle visual mode
- allow creating annotation at cursor without visual mode

### Fixed

- remove managed scrolling in agent outputs to prevent vt100 panic
- mouse scroll in agent outputs view, opencode --prompt flag
- use vt100 scrollback buffer for PTY output scrolling
- interleave comments with diff context in review prompt
- scoped diffs in agent prompt, deletion line positioning, and UI clarity
- render agent outputs from top of PTY screen, fix scrolling

### Other

- simplify PTY terminal row rendering in agent outputs

## [0.1.3](https://github.com/MutinyHQ/mdiff/compare/v0.1.2...v0.1.3) - 2026-02-18

### Added

- add confirm modal for destructive restore command
- wrap and scroll text input in all modals, add Shift+Enter for newlines

### Fixed

- always use all files and annotations for agent prompt, clear after dispatch

### Other

- cargo fmt

## [0.1.2](https://github.com/MutinyHQ/mdiff/compare/v0.1.1...v0.1.2) - 2026-02-18

### Other

- use GH_ACTION_PAT for release-plz workflow
