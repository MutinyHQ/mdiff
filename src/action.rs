use crossterm::event::KeyEvent;

/// Central action enum — all state mutations flow through here.
#[derive(Debug, Clone)]
pub enum Action {
    // Lifecycle
    Quit,
    Tick,

    // Navigation
    NavigatorUp,
    NavigatorDown,
    NavigatorTop,
    NavigatorBottom,
    SelectFile(usize),

    // Diff view
    ScrollUp,
    ScrollDown,
    ScrollToTop,
    ScrollToBottom,
    ScrollPageUp,
    ScrollPageDown,
    ToggleViewMode,
    ToggleWhitespace,

    ExpandContext,

    // Focus
    FocusNavigator,
    FocusDiffView,

    // File search (navigator)
    StartSearch,
    ConfirmSearch,
    CancelSearch,
    SearchChar(char),
    SearchBackspace,

    // Diff text search
    StartDiffSearch,
    EndDiffSearch,
    DiffSearchChar(char),
    DiffSearchBackspace,
    DiffSearchNext,
    DiffSearchPrev,

    // Git mutations
    StageFile,
    UnstageFile,
    RestoreFile,
    OpenCommitDialog,
    ConfirmCommit,
    CancelCommit,
    CommitChar(char),
    CommitBackspace,
    CommitNewline,

    // Restore confirm
    ConfirmRestore,
    CancelRestore,

    // Target change
    OpenTargetDialog,
    ConfirmTarget,
    CancelTarget,
    TargetChar(char),
    TargetBackspace,

    // Worktree
    ToggleWorktreeBrowser,
    WorktreeUp,
    WorktreeDown,
    WorktreeSelect,
    WorktreeRefresh,
    WorktreeFreeze,
    WorktreeBack,

    // Visual selection
    EnterVisualMode,
    ExitVisualMode,
    ExtendSelectionUp,
    ExtendSelectionDown,

    // Comment editor
    OpenCommentEditor,
    ConfirmComment,
    CancelComment,
    CommentChar(char),
    CommentBackspace,
    CommentNewline,
    // Annotations
    DeleteAnnotation,
    NextAnnotation,
    PrevAnnotation,
    OpenAnnotationMenu,
    AnnotationMenuUp,
    AnnotationMenuDown,
    AnnotationMenuEdit,
    AnnotationMenuDelete,
    CancelAnnotationMenu,

    // Prompt / clipboard
    CopyPromptToClipboard,
    TogglePromptPreview,

    // Agent selector
    OpenAgentSelector,
    AgentSelectorUp,
    AgentSelectorDown,
    AgentSelectorFilter(char),
    AgentSelectorBackspace,
    AgentSelectorCycleModel,
    SelectAgent,
    CancelAgentSelector,

    // Agent outputs tab
    SwitchToAgentOutputs,
    AgentOutputsUp,
    AgentOutputsDown,
    AgentOutputsCopyPrompt,
    KillAgentProcess,
    AgentOutputsSwitchWorktree,

    // PTY focus mode
    EnterPtyFocus,
    ExitPtyFocus,
    PtyInput(KeyEvent),
    PtyScrollUp,
    PtyScrollDown,

    // Review state
    ToggleFileReviewed,
    NextUnreviewed,

    // Refresh
    RefreshDiff,

    // HUD
    ToggleHud,

    // Settings modal
    OpenSettings,
    CloseSettings,
    SettingsUp,
    SettingsDown,
    SettingsLeft,
    SettingsRight,

    // Resize
    Resize,
}
