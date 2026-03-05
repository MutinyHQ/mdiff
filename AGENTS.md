# AGENTS.md

## Cursor Cloud specific instructions

**Product:** `mdiff` (crate name `mutiny-diff`) — a single-binary Rust TUI for reviewing git diffs and sending structured feedback to coding agents.

**Rust toolchain:** Pinned to `1.93.1` via `rust-toolchain.toml`. The toolchain is auto-installed by `rustup` when any `cargo` command is run.

### Build, lint, test

All commands match CI (see `.github/workflows/ci.yml`):

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Format check | `cargo fmt --all --check` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Test | `cargo test` |
| Run | `cargo run` (or `cargo run -- <args>`) |

### Running the application

`mdiff` is a TUI that must be run inside a git repository with a real terminal (not a pipe). It diffs the working tree against HEAD by default. To test it you need uncommitted changes or pass a target ref (e.g. `cargo run -- main`).

### Native dependencies

The build requires `cmake`, `pkg-config`, and a C compiler (`cc`/`gcc`). These are pre-installed in the Cloud VM. The `git2` crate vendors OpenSSL and libgit2, so no system OpenSSL dev headers are needed.

### Clipboard

The `arboard` clipboard crate requires a display server or clipboard provider. On the headless Cloud VM, clipboard operations (`y` to copy prompt) will fail gracefully — this does not affect core diff review functionality.
