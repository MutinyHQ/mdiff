mod action;
mod agent_runner;
mod app;
mod async_diff;
mod cli;
mod components;
mod config;
mod context;
mod display_map;
mod event;
mod git;
mod highlight;
mod session;
mod state;
mod template;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::env;

use crate::app::{parse_target, App};
use crate::cli::Cli;
use crate::git::RepoCache;
use crate::state::DiffOptions;

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
    let diff_options = DiffOptions::new(cli.ignore_whitespace, cli.unified);
    let mut app = App::new(diff_options, cli.worktree_browser, target, repo_path);

    let mut terminal = tui::init()?;
    let result = app.run(&mut terminal).await;
    tui::restore()?;

    if let Err(ref e) = result {
        eprintln!("mdiff: {e:#}");
    }

    result
}
