use crossterm::event::KeyEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitCombo {
    CtrlC,
    CtrlD,
}

impl QuitCombo {
    pub fn label(self) -> &'static str {
        match self {
            Self::CtrlC => "Ctrl+C",
            Self::CtrlD => "Ctrl+D",
        }
    }
}

/// Central action enum — all state mutations flow through here.
#[derive(Debug, Clone)]
pub enum Action {
    // Lifecycle
    Quit,
    ConfirmQuitSignal(QuitCombo),
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

    // Hunk navigation
    JumpNextHunk,
    JumpPrevHunk,

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
    PtyPaste(String),
    PtyScrollUp,
    PtyScrollDown,

    // Review state
    ToggleFileReviewed,
    NextUnreviewed,

    // Refresh
    RefreshDiff,

    // HUD
    #[allow(dead_code)]
    ToggleHud,

    // Which-key help overlay
    ToggleWhichKey,

    // Settings modal
    OpenSettings,
    CloseSettings,
    SettingsUp,
    SettingsDown,
    SettingsLeft,
    SettingsRight,

    // Generic text input navigation
    TextCursorLeft,
    TextCursorRight,
    TextCursorHome,
    TextCursorEnd,
    TextDeleteWord,

    // Resize
    Resize,
}
