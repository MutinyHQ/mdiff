# mdiff

A terminal UI for reviewing git diffs — with syntax highlighting, split/unified views, and built-in git operations.

## Features

- **Split and unified diff views** — side-by-side or interleaved, toggle with `Tab`
- **Syntax highlighting** — tree-sitter powered, supports Rust, TypeScript, JavaScript, Python, Go, Ruby, JSON, TOML, YAML, CSS, HTML, and Bash
- **Git operations** — stage, unstage, restore files, and commit without leaving the TUI
- **Annotations** — add inline comments on diff lines for coding agents to act on *(coming soon)*
- **Worktree browser** — browse and switch between git worktrees, with automatic detection of active coding agents (Claude Code, Cursor, Aider, Copilot)
- **Fuzzy file search** — quickly filter the file list with `/`
- **Whitespace toggle** — hide whitespace-only changes with `w`

## Installation

**Homebrew**

```
brew install mutinyhq/tap/mdiff
```

**Cargo**

```
cargo install mutiny-diff
```

**GitHub Releases**

Pre-built binaries for macOS (Intel & Apple Silicon) and Linux (x86_64 & ARM) are available on the [releases page](https://github.com/mutinyhq/mdiff/releases).

## Usage

```bash
# Diff HEAD vs working directory
mdiff

# Diff against a branch
mdiff main

# Diff against a specific commit
mdiff abc1234

# Open the worktree browser
mdiff --wt

# Start in unified view, ignoring whitespace
mdiff --unified -w
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+C` | Quit |
| `Tab` | Toggle split/unified view |
| `w` | Toggle whitespace |
| `/` | Search files |
| `s` | Stage file |
| `u` | Unstage file |
| `r` | Restore file |
| `c` | Open commit dialog |
| `Ctrl+W` | Toggle worktree browser |

### File navigator

| Key | Action |
|-----|--------|
| `j` / `↓` | Next file |
| `k` / `↑` | Previous file |
| `l` / `→` / `Enter` | Focus diff view |

### Diff view

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `h` / `←` | Focus file navigator |
| `PageUp` | Scroll page up |
| `PageDown` | Scroll page down |

### Worktree browser

| Key | Action |
|-----|--------|
| `j` / `↓` | Next worktree |
| `k` / `↑` | Previous worktree |
| `Enter` | Select worktree |
| `r` | Refresh list |
| `f` | Freeze worktree (stage all + auto-commit) |
| `Esc` | Back to diff view |

## CLI Reference

| Flag | Description |
|------|-------------|
| `<TARGET>` | Branch, commit, or ref to diff against (default: HEAD) |
| `--wt` | Open worktree browser directly |
| `-w`, `--ignore-ws` | Ignore whitespace changes |
| `--unified` | Start in unified view instead of split |

## License

MIT
