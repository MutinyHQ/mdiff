# mdiff

A terminal UI for reviewing git diffs — with syntax highlighting, split/unified views, inline annotations, and agent-assisted code review.

## Features

- **Split and unified diff views** — side-by-side or interleaved, toggle with `Tab`
- **Syntax highlighting** — tree-sitter powered, supports Rust, TypeScript, JavaScript, Python, Go, Ruby, JSON, TOML, YAML, CSS, HTML, and Bash
- **Git operations** — stage, unstage, restore files, and commit without leaving the TUI
- **Inline annotations** — select diff lines in visual mode and attach review comments that persist across sessions
- **Agent handoff** — launch a configured coding agent with a templated prompt containing the selected code, diff context, and your annotations; or copy the prompt to your clipboard for use in any existing agent session
- **Runtime target switching** — change the comparison ref (branch, tag, commit) at runtime with `t`
- **Worktree browser** — browse and switch between git worktrees, with automatic detection of active coding agents (Claude Code, Codex, OpenCode, Gemini)
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

## Workflow: Annotate and Hand Off

1. Run `mdiff` to review your diff
2. Navigate to a file, press `v` to enter visual mode, and select the lines you want to comment on
3. Press `i` to open the comment editor and describe the change you want — a bug fix, a refactor, a question
4. Repeat for as many files and regions as needed; annotations are saved per-target and persist across sessions
5. When ready, either:
   - Press `Ctrl+A` to pick a configured agent, which launches with a templated prompt containing the selected code, surrounding context, and all your annotations for that file
   - Press `y` to copy the rendered prompt to your clipboard, then paste it into any existing agent session (Claude Code, Cursor, Copilot, etc.)
6. Press `p` to preview the rendered prompt before sending

## Keybindings

Press `?` to show all keybindings in the HUD. The essentials:

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Next item / scroll down |
| `k` / `↑` | Previous item / scroll up |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `h` / `←` | Focus file navigator |
| `l` / `→` / `Enter` | Focus diff view |
| `/` | Search files |
| `Tab` | Toggle split/unified view |
| `PageUp` / `PageDown` | Scroll page |

### Annotations & Prompts

| Key | Action |
|-----|--------|
| `v` | Enter visual mode (select lines) |
| `i` | Add comment on selection |
| `a` | Open annotation menu on current line |
| `d` | Delete annotation on selection |
| `]` / `[` | Jump to next/previous annotation |
| `y` | Copy rendered prompt to clipboard |
| `p` | Toggle prompt preview |
| `Ctrl+A` | Open agent selector |

### Git Operations

| Key | Action |
|-----|--------|
| `s` | Stage file |
| `u` | Unstage file |
| `r` | Restore file |
| `c` | Open commit dialog |
| `t` | Change comparison target |
| `R` | Refresh diff |

### General

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+C` / `Ctrl+D` | Quit (from any modal) |
| `w` | Toggle whitespace |
| `o` | Toggle agent outputs tab |
| `Ctrl+W` | Toggle worktree browser |
| `?` | Show/hide all keybindings |

### Worktree Browser

| Key | Action |
|-----|--------|
| `j` / `↓` | Next worktree |
| `k` / `↑` | Previous worktree |
| `Enter` | Select worktree |
| `r` | Refresh list |
| `f` | Freeze worktree (stage all + auto-commit) |
| `Esc` | Back to diff view |

## Agent Configuration

Configure agents in `~/.config/mdiff/config.toml`:

```toml
[[agents]]
name = "claude"
command = "claude --model {model} --print '{rendered_prompt}'"
models = ["sonnet", "opus"]

[[agents]]
name = "codex"
command = "codex --model {model} --prompt '{rendered_prompt}'"
models = ["o3-mini", "o4-mini"]
```

The `{rendered_prompt}` placeholder is replaced with the templated prompt containing the diff context, selected code, and your annotations. The `{model}` placeholder is replaced with the model you select.

## CLI Reference

| Flag | Description |
|------|-------------|
| `<TARGET>` | Branch, commit, or ref to diff against (default: HEAD) |
| `--wt` | Open worktree browser directly |
| `-w`, `--ignore-ws` | Ignore whitespace changes |
| `--unified` | Start in unified view instead of split |

## License

MIT
