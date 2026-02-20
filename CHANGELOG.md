# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
