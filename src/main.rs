mod action;
#[allow(dead_code)]
mod agent_runner;
mod app;
mod async_diff;
mod cli;
mod components;
mod config;
mod display_map;
mod event;
mod git;
mod highlight;
mod pty_runner;
mod session;
mod state;
mod theme;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::env;

use crate::app::{parse_target, App};
use crate::cli::Cli;
use crate::git::RepoCache;
use crate::state::DiffOptions;
use crate::theme::Theme;

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Best-effort terminal restore so the user gets their shell back
        let _ = tui::restore();
        default_hook(panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().ok();
    install_panic_hook();

    let cli = Cli::parse();

    let cwd = env::current_dir()?;

    // Validate we're in a git repo before launching TUI
    let repo = match RepoCache::open(&cwd) {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "mdiff: not a git repository (or any parent up to mount point /)\n\
                 Run this command from inside a git working tree."
            );
            std::process::exit(1);
        }
    };
    let repo_path = repo.workdir().to_path_buf();
    drop(repo);

    let target = parse_target(cli.target.as_deref());

    // Load config, apply CLI overrides
    let mut config = config::load_config();
    if let Some(ref theme_name) = cli.theme {
        config.theme = Theme::from_name(theme_name);
    }

    // Merge CLI flags with config-file settings (CLI wins)
    let unified = cli.unified || config.unified.unwrap_or(false);
    let ignore_ws = cli.ignore_whitespace || config.ignore_whitespace.unwrap_or(false);
    let context_lines = config.context_lines;

    let diff_options = DiffOptions::new(ignore_ws, unified);
    let mut app = App::new(
        diff_options,
        cli.worktree_browser,
        target,
        repo_path,
        config,
        context_lines,
    );

    let mut terminal = tui::init()?;
    let result = app.run(&mut terminal).await;
    tui::restore()?;

    if let Err(ref e) = result {
        eprintln!("mdiff: {e:#}");
    }

    result
}
