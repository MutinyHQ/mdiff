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
    Resize(u16, u16),
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
                    Some(Ok(CrosstermEvent::Resize(w, h))) => {
                        if event_tx.send(Event::Resize(w, h)).is_err() {
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

/// Map a key event to an action based on current app context.
pub fn map_key_to_action(
    key: KeyEvent,
    focus: FocusPanel,
    search_active: bool,
    commit_dialog_open: bool,
    active_view: ActiveView,
) -> Option<Action> {
    // Commit dialog mode
    if commit_dialog_open {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelCommit),
            KeyCode::Enter => Some(Action::ConfirmCommit),
            KeyCode::Backspace => Some(Action::CommitBackspace),
            KeyCode::Char(c) => Some(Action::CommitChar(c)),
            _ => None,
        };
    }

    // Search mode
    if search_active {
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

    // Global bindings
    match key.code {
        KeyCode::Char('q') => return Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::Quit)
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::ToggleWorktreeBrowser)
        }
        _ => {}
    }

    // Worktree browser mode
    if active_view == ActiveView::WorktreeBrowser {
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

    // Diff explorer global bindings
    match key.code {
        KeyCode::Tab => return Some(Action::ToggleViewMode),
        KeyCode::Char('w') => return Some(Action::ToggleWhitespace),
        KeyCode::Char('/') => return Some(Action::StartSearch),
        KeyCode::Char('s') => return Some(Action::StageFile),
        KeyCode::Char('u') => return Some(Action::UnstageFile),
        KeyCode::Char('r') => return Some(Action::RestoreFile),
        KeyCode::Char('c') => return Some(Action::OpenCommitDialog),
        _ => {}
    }

    // Focus-dependent bindings
    match focus {
        FocusPanel::Navigator => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::NavigatorUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::NavigatorDown),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => Some(Action::FocusDiffView),
            _ => None,
        },
        FocusPanel::DiffView => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::FocusNavigator),
            KeyCode::PageUp => Some(Action::ScrollPageUp),
            KeyCode::PageDown => Some(Action::ScrollPageDown),
            _ => None,
        },
    }
}
