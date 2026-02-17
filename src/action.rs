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
    AgentOutputsScrollUp,
    AgentOutputsScrollDown,
    AgentOutputsCopyPrompt,
    KillAgentProcess,

    // Resize
    Resize,
}
