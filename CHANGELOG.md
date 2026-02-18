# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/MutinyHQ/mdiff/compare/v0.1.0...v0.1.1) - 2026-02-18

### Added

- add theme system with 6 built-in themes and settings modal
- wrap HUD bindings and auto-collapse on action
- add expandable context lines with per-section expansion
- collapse HUD to essential bindings, expand with ? key
- add g/G shortcuts for top/bottom navigation
- add line wrapping in diff view for long lines
- add runtime comparison target switching via t key
- add release-plz for automated versioning and changelogs

### Fixed

- make Ctrl-C and Ctrl-D quit from any modal or input

### Other

- cargo fmt
- rewrite README for developer feedback workflow with coding agents
- Bump version to 0.1.1
- Fix agent CLI commands to enable non-interactive file edits
- Sort worktrees by most recently updated first
- Fix cargo fmt in cli.rs
- Add `g` key to refresh diff from filesystem
- Add --version flag, update README agent list
