use crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyModifiers, MouseEvent,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::action::Action;
use crate::state::app_state::{ActiveView, FocusPanel};

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize,
    Tick,
}

pub struct EventReader {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventReader {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let event_tx = tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            loop {
                match reader.next().await {
                    Some(Ok(CrosstermEvent::Key(key))) => {
                        if event_tx.send(Event::Key(key)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(CrosstermEvent::Mouse(mouse))) => {
                        if event_tx.send(Event::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(CrosstermEvent::Resize(_, _))) => {
                        if event_tx.send(Event::Resize).is_err() {
                            break;
                        }
                    }
                    Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        });

        let tick_tx = tx;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                if tick_tx.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// Non-blocking: returns a pending event if one is available, or None.
    pub fn try_next(&mut self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
}

/// All context needed to map a key event to an action.
pub struct KeyContext {
    pub focus: FocusPanel,
    pub search_active: bool,
    pub commit_dialog_open: bool,
    pub target_dialog_open: bool,
    pub comment_editor_open: bool,
    pub agent_selector_open: bool,
    pub annotation_menu_open: bool,
    pub restore_confirm_open: bool,
    pub settings_open: bool,
    pub visual_mode_active: bool,
    pub active_view: ActiveView,
}

/// Map a key event to an action based on current app context.
pub fn map_key_to_action(key: KeyEvent, ctx: &KeyContext) -> Option<Action> {
    // Priority 0: Ctrl-C / Ctrl-D always quit, even inside modals
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('d') => return Some(Action::Quit),
            _ => {}
        }
    }

    // Priority 0.5: Restore confirm dialog
    if ctx.restore_confirm_open {
        return match key.code {
            KeyCode::Enter | KeyCode::Char('y') => Some(Action::ConfirmRestore),
            KeyCode::Esc | KeyCode::Char('n') => Some(Action::CancelRestore),
            _ => None,
        };
    }

    // Priority 1: Commit dialog mode
    if ctx.commit_dialog_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelCommit),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::CommitNewline)
            }
            KeyCode::Enter => Some(Action::ConfirmCommit),
            KeyCode::Backspace => Some(Action::CommitBackspace),
            KeyCode::Char(c) => Some(Action::CommitChar(c)),
            _ => None,
        };
    }

    // Priority 1.5: Target dialog mode
    if ctx.target_dialog_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelTarget),
            KeyCode::Enter => Some(Action::ConfirmTarget),
            KeyCode::Backspace => Some(Action::TargetBackspace),
            KeyCode::Char(c) => Some(Action::TargetChar(c)),
            _ => None,
        };
    }

    // Priority 2: Comment editor mode
    if ctx.comment_editor_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelComment),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::CommentNewline)
            }
            KeyCode::Enter => Some(Action::ConfirmComment),
            KeyCode::Backspace => Some(Action::CommentBackspace),
            KeyCode::Char(c) => Some(Action::CommentChar(c)),
            _ => None,
        };
    }

    // Priority 2.3: Settings modal
    if ctx.settings_open {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::SettingsDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::SettingsUp),
            KeyCode::Char('h') | KeyCode::Left => Some(Action::SettingsLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::SettingsRight),
            KeyCode::Esc | KeyCode::Char(':') => Some(Action::CloseSettings),
            _ => None,
        };
    }

    // Priority 2.5: Agent selector mode
    if ctx.agent_selector_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelAgentSelector),
            KeyCode::Enter => Some(Action::SelectAgent),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::AgentSelectorUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::AgentSelectorDown),
            KeyCode::Tab => Some(Action::AgentSelectorCycleModel),
            KeyCode::Backspace => Some(Action::AgentSelectorBackspace),
            KeyCode::Char(c) => Some(Action::AgentSelectorFilter(c)),
            _ => None,
        };
    }

    // Priority 2.75: Annotation menu mode
    if ctx.annotation_menu_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelAnnotationMenu),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::AnnotationMenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::AnnotationMenuDown),
            KeyCode::Char('e') | KeyCode::Enter => Some(Action::AnnotationMenuEdit),
            KeyCode::Char('d') => Some(Action::AnnotationMenuDelete),
            _ => None,
        };
    }

    // Priority 3: Search mode
    if ctx.search_active {
        return match key.code {
            KeyCode::Esc => Some(Action::EndSearch),
            KeyCode::Enter => Some(Action::EndSearch),
            KeyCode::Backspace => Some(Action::SearchBackspace),
            KeyCode::Char(c) => Some(Action::SearchChar(c)),
            KeyCode::Up => Some(Action::NavigatorUp),
            KeyCode::Down => Some(Action::NavigatorDown),
            _ => None,
        };
    }

    // Priority 4: Global bindings (always active)
    match key.code {
        KeyCode::Char('q') if !ctx.visual_mode_active => return Some(Action::Quit),
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::ToggleWorktreeBrowser)
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::OpenAgentSelector)
        }
        _ => {}
    }

    // Annotation navigation (global in DiffExplorer)
    if ctx.active_view == ActiveView::DiffExplorer {
        match key.code {
            KeyCode::Char(']') => return Some(Action::NextAnnotation),
            KeyCode::Char('[') => return Some(Action::PrevAnnotation),
            _ => {}
        }
    }

    // Priority 5: Worktree browser mode
    if ctx.active_view == ActiveView::WorktreeBrowser {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::WorktreeUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::WorktreeDown),
            KeyCode::Enter => Some(Action::WorktreeSelect),
            KeyCode::Char('r') => Some(Action::WorktreeRefresh),
            KeyCode::Char('f') => Some(Action::WorktreeFreeze),
            KeyCode::Esc => Some(Action::WorktreeBack),
            _ => None,
        };
    }

    // Priority 5.5: Agent outputs tab
    if ctx.active_view == ActiveView::AgentOutputs {
        // Check Ctrl+K first (before plain 'k')
        if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Some(Action::KillAgentProcess);
        }
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::AgentOutputsUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::AgentOutputsDown),
            KeyCode::Char('K') => Some(Action::AgentOutputsScrollUp),
            KeyCode::Char('J') => Some(Action::AgentOutputsScrollDown),
            KeyCode::Char('y') => Some(Action::AgentOutputsCopyPrompt),
            KeyCode::Esc => Some(Action::SwitchToAgentOutputs), // toggle back
            _ => None,
        };
    }

    // Priority 6: Diff explorer global bindings
    match key.code {
        KeyCode::Tab => return Some(Action::ToggleViewMode),
        KeyCode::Char('w') if !ctx.visual_mode_active => return Some(Action::ToggleWhitespace),

        KeyCode::Char('/') => return Some(Action::StartSearch),
        KeyCode::Char('s') if !ctx.visual_mode_active => return Some(Action::StageFile),
        KeyCode::Char('u') if !ctx.visual_mode_active => return Some(Action::UnstageFile),
        KeyCode::Char('r') if !ctx.visual_mode_active => return Some(Action::RestoreFile),
        KeyCode::Char('c') if !ctx.visual_mode_active => return Some(Action::OpenCommitDialog),
        KeyCode::Char('o') if !ctx.visual_mode_active => return Some(Action::SwitchToAgentOutputs),
        KeyCode::Char('R') => return Some(Action::RefreshDiff),
        KeyCode::Char('t') if !ctx.visual_mode_active => return Some(Action::OpenTargetDialog),
        KeyCode::Char('?') => return Some(Action::ToggleHud),
        KeyCode::Char(':') if !ctx.visual_mode_active => return Some(Action::OpenSettings),
        _ => {}
    }

    // Priority 7: Visual mode in DiffView
    if ctx.visual_mode_active && ctx.focus == FocusPanel::DiffView {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::ExtendSelectionUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::ExtendSelectionDown),
            KeyCode::Char('i') => Some(Action::OpenCommentEditor),
            KeyCode::Char('d') => Some(Action::DeleteAnnotation),
            KeyCode::Char('y') => Some(Action::CopyPromptToClipboard),
            KeyCode::Char('v') | KeyCode::Char('V') | KeyCode::Esc => Some(Action::ExitVisualMode),
            _ => None,
        };
    }

    // Priority 8: Focus-dependent bindings
    match ctx.focus {
        FocusPanel::Navigator => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::NavigatorUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::NavigatorDown),
            KeyCode::Char('g') => Some(Action::NavigatorTop),
            KeyCode::Char('G') => Some(Action::NavigatorBottom),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => Some(Action::FocusDiffView),
            _ => None,
        },
        FocusPanel::DiffView => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
            KeyCode::Char('g') => Some(Action::ScrollToTop),
            KeyCode::Char('G') => Some(Action::ScrollToBottom),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::FocusNavigator),
            KeyCode::PageUp => Some(Action::ScrollPageUp),
            KeyCode::PageDown => Some(Action::ScrollPageDown),
            KeyCode::Char(' ') => Some(Action::ExpandContext),
            KeyCode::Char('v') | KeyCode::Char('V') => Some(Action::EnterVisualMode),
            KeyCode::Char('i') => Some(Action::OpenCommentEditor),
            KeyCode::Char('p') => Some(Action::TogglePromptPreview),
            KeyCode::Char('y') => Some(Action::CopyPromptToClipboard),
            KeyCode::Char('a') => Some(Action::OpenAnnotationMenu),
            _ => None,
        },
    }
}
