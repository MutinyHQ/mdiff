use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mdiff",
    version,
    about = "TUI git diff viewer with worktree management"
)]
pub struct Cli {
    /// Target to diff against (branch, commit, or ref)
    pub target: Option<String>,

    /// Open worktree browser directly
    #[arg(long = "wt")]
    pub worktree_browser: bool,

    /// Ignore whitespace changes
    #[arg(short = 'w', long = "ignore-ws")]
    pub ignore_whitespace: bool,

    /// Start in unified (consolidated) view instead of split
    #[arg(long)]
    pub unified: bool,
}
