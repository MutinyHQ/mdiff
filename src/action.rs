/// Central action enum — all state mutations flow through here.
#[derive(Debug, Clone)]
pub enum Action {
    // Lifecycle
    Quit,
    Tick,

    // Navigation
    NavigatorUp,
    NavigatorDown,
    SelectFile(usize),

    // Diff view
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ToggleViewMode,
    ToggleWhitespace,

    // Focus
    FocusNavigator,
    FocusDiffView,

    // Search
    StartSearch,
    EndSearch,
    SearchChar(char),
    SearchBackspace,

    // Git mutations
    StageFile,
    UnstageFile,
    RestoreFile,
    OpenCommitDialog,
    ConfirmCommit,
    CancelCommit,
    CommitChar(char),
    CommitBackspace,

    // Worktree
    ToggleWorktreeBrowser,
    WorktreeUp,
    WorktreeDown,
    WorktreeSelect,
    WorktreeRefresh,
    WorktreeFreeze,
    WorktreeBack,

    // Resize
    Resize,
}
