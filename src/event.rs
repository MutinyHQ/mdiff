use crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use futures::StreamExt;
use ratatui::layout::Rect;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::action::Action;
use crate::action::QuitCombo;
use crate::state::annotation_state::{AnnotationCategory, AnnotationSeverity};
use crate::state::app_state::{ActiveView, CategoryPickerPhase, FocusPanel};
use crate::state::navigator_state::NavigatorEntry;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
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
                        // Only forward key press events, not release/repeat
                        if key.kind == KeyEventKind::Press
                            && event_tx.send(Event::Key(key)).is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(CrosstermEvent::Mouse(mouse))) => {
                        if event_tx.send(Event::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(CrosstermEvent::Paste(text))) => {
                        if event_tx.send(Event::Paste(text)).is_err() {
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
    pub diff_search_active: bool,
    pub global_search_active: bool,
    pub commit_dialog_open: bool,
    pub target_dialog_open: bool,
    pub comment_editor_open: bool,
    pub category_picker_open: bool,
    pub category_picker_phase: CategoryPickerPhase,
    pub agent_selector_open: bool,
    pub annotation_menu_open: bool,
    pub restore_confirm_open: bool,
    pub settings_open: bool,
    pub visual_mode_active: bool,
    pub active_view: ActiveView,
    pub pty_focus: bool,
    pub checklist_panel_open: bool,
    pub bookmark_list_open: bool,
    pub which_key_visible: bool,
    pub tree_mode: bool,
    pub tree_z_pending: bool,
    pub bracket_pending: Option<char>,
    pub mark_pending: bool,
    pub jump_mark_pending: bool,
    pub command_bar_active: bool,
    pub file_picker_active: bool,
    pub agentic_review_modal_open: bool,
    pub agentic_review_panel_open: bool,
    pub agentic_review_composing: bool,
    pub window_pending: bool,
    pub stats_dashboard_open: bool,
    pub timeline_active: bool,
}

/// Context for mouse event mapping.
pub struct MouseContext<'a> {
    pub navigator_rect: Rect,
    pub diff_view_rect: Rect,
    pub navigator_scroll_offset: usize,
    pub navigator_item_count: usize,
    pub navigator_visible_entries: &'a [(usize, &'a NavigatorEntry)],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Navigator,
    DiffView,
}

impl MouseContext<'_> {
    /// Determine which panel a screen coordinate falls in.
    fn panel_at(&self, col: u16, row: u16) -> Option<Panel> {
        if self.navigator_rect.x <= col
            && col < self.navigator_rect.x + self.navigator_rect.width
            && self.navigator_rect.y <= row
            && row < self.navigator_rect.y + self.navigator_rect.height
        {
            Some(Panel::Navigator)
        } else if self.diff_view_rect.x <= col
            && col < self.diff_view_rect.x + self.diff_view_rect.width
            && self.diff_view_rect.y <= row
            && row < self.diff_view_rect.y + self.diff_view_rect.height
        {
            Some(Panel::DiffView)
        } else {
            None
        }
    }

    /// Convert a mouse row in the navigator area to a visible entry index.
    fn navigator_row_to_visible_index(&self, row: u16) -> Option<usize> {
        let relative_row = row.saturating_sub(self.navigator_rect.y + 1); // +1 for border
        let visible_index = self.navigator_scroll_offset + relative_row as usize;
        if visible_index < self.navigator_item_count {
            Some(visible_index)
        } else {
            None
        }
    }
}

/// Map a key event to an action based on current app context.
pub fn map_key_to_action(key: KeyEvent, ctx: &KeyContext) -> Option<Action> {
    let review_composing = ctx.focus == FocusPanel::ReviewPanel && ctx.agentic_review_composing;

    // Priority 0: Window prefix mode (Ctrl+W + next key) must win even when PTY
    // focus is active, otherwise the follow-up key gets swallowed by the PTY.
    if ctx.window_pending {
        // Ctrl+W + h/l navigates left/right through the horizontal pane layout:
        // Navigator ← → DiffView ← → ReviewPanel/ChecklistPanel
        return match key.code {
            KeyCode::Char('h') | KeyCode::Left => Some(Action::FocusPaneLeft),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::FocusPaneRight),
            KeyCode::Char('w') => Some(Action::CycleFocus),
            KeyCode::Char('o') => Some(Action::SwitchToAgentOutputs),
            KeyCode::Char('b') => Some(Action::ToggleWorktreeBrowser),
            KeyCode::Esc => None, // Cancel prefix
            _ => None,
        };
    }

    // Priority 0.1: PTY focus mode — forward ALL keys except Ctrl+W to the PTY.
    // Use Ctrl+W to switch away from the PTY panel (no Esc trap needed).
    if ctx.pty_focus {
        if key.code == KeyCode::Char('w') && key.modifiers.contains(KeyModifiers::CONTROL) {
            // Fall through to window prefix handling below
        } else {
            return Some(Action::PtyInput(key));
        }
    }

    // Priority 0.5: Ctrl-C / Ctrl-D quit (not in PTY focus)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('d') => {
                return Some(Action::ConfirmQuitSignal(match key.code {
                    KeyCode::Char('c') => QuitCombo::CtrlC,
                    KeyCode::Char('d') => QuitCombo::CtrlD,
                    _ => unreachable!(),
                }));
            }
            _ => {}
        }
    }

    // Priority 0.75: Restore confirm dialog
    if ctx.restore_confirm_open {
        return match key.code {
            KeyCode::Enter | KeyCode::Char('y') => Some(Action::ConfirmRestore),
            KeyCode::Esc | KeyCode::Char('n') => Some(Action::CancelRestore),
            _ => None,
        };
    }

    // Priority 1: Commit dialog mode
    if ctx.commit_dialog_open {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::CancelCommit),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::CommitNewline)
            }
            KeyCode::Enter => Some(Action::ConfirmCommit),
            KeyCode::Backspace => Some(Action::CommitBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::CommitChar(c)),
            _ => None,
        };
    }

    // Priority 1.5: Target dialog mode
    if ctx.target_dialog_open {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::CancelTarget),
            KeyCode::Enter => Some(Action::ConfirmTarget),
            KeyCode::Backspace => Some(Action::TargetBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::TargetChar(c)),
            _ => None,
        };
    }

    // Priority 2.1: Category/severity picker
    if ctx.category_picker_open {
        match ctx.category_picker_phase {
            CategoryPickerPhase::SelectCategory => {
                return match key.code {
                    KeyCode::Char('b') => Some(Action::SelectCategory(AnnotationCategory::Bug)),
                    KeyCode::Char('s') => Some(Action::SelectCategory(AnnotationCategory::Style)),
                    KeyCode::Char('p') => {
                        Some(Action::SelectCategory(AnnotationCategory::Performance))
                    }
                    KeyCode::Char('x') => {
                        Some(Action::SelectCategory(AnnotationCategory::Security))
                    }
                    KeyCode::Char('g') => {
                        Some(Action::SelectCategory(AnnotationCategory::Suggestion))
                    }
                    KeyCode::Char('q') => {
                        Some(Action::SelectCategory(AnnotationCategory::Question))
                    }
                    KeyCode::Char('n') => Some(Action::SelectCategory(AnnotationCategory::Nitpick)),
                    KeyCode::Enter => Some(Action::CategoryPickerDefault),
                    KeyCode::Esc => Some(Action::CancelCategoryPicker),
                    _ => None,
                };
            }
            CategoryPickerPhase::SelectSeverity => {
                return match key.code {
                    KeyCode::Char('c') => {
                        Some(Action::SelectSeverity(AnnotationSeverity::Critical))
                    }
                    KeyCode::Char('M') => Some(Action::SelectSeverity(AnnotationSeverity::Major)),
                    KeyCode::Char('m') => Some(Action::SelectSeverity(AnnotationSeverity::Minor)),
                    KeyCode::Char('i') => Some(Action::SelectSeverity(AnnotationSeverity::Info)),
                    KeyCode::Enter => Some(Action::SelectSeverity(AnnotationSeverity::Minor)),
                    KeyCode::Esc => Some(Action::CancelCategoryPicker),
                    _ => None,
                };
            }
        }
    }

    // Priority 2: Comment editor mode
    if ctx.comment_editor_open {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::CancelComment),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Action::CommentNewline)
            }
            KeyCode::Enter => Some(Action::ConfirmComment),
            KeyCode::Backspace => Some(Action::CommentBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::CommentChar(c)),
            _ => None,
        };
    }

    // (Agentic review composing is now handled at Priority 8 under FocusPanel::ReviewPanel)

    // Priority 2.2: Command bar mode
    if ctx.command_bar_active {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::CommandBarCancel),
            KeyCode::Enter => Some(Action::CommandBarConfirm),
            KeyCode::Backspace => Some(Action::CommandBarBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::CommandBarChar(c)),
            _ => None,
        };
    }

    // Priority 2.25: File picker mode
    if ctx.file_picker_active {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                KeyCode::Char('j') => Some(Action::FilePickerDown),
                KeyCode::Char('k') => Some(Action::FilePickerUp),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::FilePickerCancel),
            KeyCode::Enter => Some(Action::FilePickerConfirm),
            KeyCode::Up => Some(Action::FilePickerUp),
            KeyCode::Down => Some(Action::FilePickerDown),
            KeyCode::Backspace => Some(Action::FilePickerBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::FilePickerChar(c)),
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

    // Priority 2.35: Bookmark list panel navigation
    if ctx.bookmark_list_open {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::BookmarkListDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::BookmarkListUp),
            KeyCode::Enter => Some(Action::BookmarkListSelect),
            KeyCode::Char('d') => Some(Action::BookmarkListDelete),
            KeyCode::Esc => Some(Action::ToggleBookmarkList),
            _ => None,
        };
    }

    // Priority 2.37: Stats dashboard overlay
    if ctx.stats_dashboard_open {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('S') => Some(Action::ToggleStatsDashboard),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::StatsDashboardUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::StatsDashboardDown),
            KeyCode::Enter => Some(Action::StatsDashboardSelect),
            KeyCode::Char('s') => Some(Action::StatsDashboardSort),
            _ => None,
        };
    }

    // Priority 2.4: Checklist panel navigation
    if ctx.checklist_panel_open {
        return match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::ChecklistDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::ChecklistUp),
            KeyCode::Char(' ') | KeyCode::Enter => Some(Action::ChecklistToggleItem),
            KeyCode::Char('n') => Some(Action::ChecklistAddNote),
            KeyCode::Esc => Some(Action::ToggleChecklist), // Close panel
            _ => None,
        };
    }

    // Priority 2.5: Agent selector mode
    if ctx.agent_selector_open {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::CancelAgentSelector),
            KeyCode::Enter => Some(Action::SelectAgent),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::AgentSelectorUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::AgentSelectorDown),
            KeyCode::Tab => Some(Action::AgentSelectorCycleModel),
            KeyCode::Backspace => Some(Action::AgentSelectorBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
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

    // Priority 2.8: Global diff search mode
    if ctx.global_search_active {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                KeyCode::Char('n') => Some(Action::GlobalSearchNext),
                KeyCode::Char('p') => Some(Action::GlobalSearchPrev),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::EndGlobalSearch),
            KeyCode::Enter => Some(Action::GlobalSearchNext),
            KeyCode::Backspace => Some(Action::GlobalSearchBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::GlobalSearchChar(c)),
            _ => None,
        };
    }

    // Priority 3: Diff text search mode
    if ctx.diff_search_active {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc | KeyCode::Enter => Some(Action::EndDiffSearch),
            KeyCode::Backspace => Some(Action::DiffSearchBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::DiffSearchChar(c)),
            _ => None,
        };
    }

    // Priority 3.5: File search mode (navigator)
    if ctx.search_active {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('a') => Some(Action::TextCursorHome),
                KeyCode::Char('e') => Some(Action::TextCursorEnd),
                KeyCode::Char('w') => Some(Action::TextDeleteWord),
                _ => None,
            };
        }
        return match key.code {
            KeyCode::Esc => Some(Action::CancelSearch),
            KeyCode::Enter => Some(Action::ConfirmSearch),
            KeyCode::Backspace => Some(Action::SearchBackspace),
            KeyCode::Left => Some(Action::TextCursorLeft),
            KeyCode::Right => Some(Action::TextCursorRight),
            KeyCode::Home => Some(Action::TextCursorHome),
            KeyCode::End => Some(Action::TextCursorEnd),
            KeyCode::Char(c) => Some(Action::SearchChar(c)),
            KeyCode::Up => Some(Action::NavigatorUp),
            KeyCode::Down => Some(Action::NavigatorDown),
            _ => None,
        };
    }

    // Priority 3.75: Which-key overlay - any key closes the overlay
    if ctx.which_key_visible {
        return Some(Action::ToggleWhichKey);
    }

    // Priority 3.8: Agentic review panel scroll (when panel is visible)
    if ctx.agentic_review_panel_open && !ctx.agentic_review_modal_open {
        match key.code {
            KeyCode::Char('{') => return Some(Action::AgenticReviewPanelUp),
            KeyCode::Char('}') => return Some(Action::AgenticReviewPanelDown),
            _ => {}
        }
    }

    // Priority 3.9: Timeline scrubber navigation (when timeline is active and in DiffExplorer)
    if ctx.timeline_active && ctx.active_view == ActiveView::DiffExplorer && !review_composing {
        match key.code {
            KeyCode::Char('>') | KeyCode::Char('.') => return Some(Action::TimelineNext),
            KeyCode::Char('<') | KeyCode::Char(',') => return Some(Action::TimelinePrev),
            _ => {}
        }
    }

    // Priority 4: Global bindings (always active)
    match key.code {
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::WindowPrefix)
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::OpenAgentSelector)
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::StartGlobalSearch)
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::ExportFeedback)
        }
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::OpenFilePicker)
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::OpenAgenticReview)
        }
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::ToggleTimeline)
        }
        _ => {}
    }

    // Priority 4.1: Pending bracket sequence (]b / [b for bookmark nav)
    if let Some(bracket) = ctx.bracket_pending {
        if ctx.active_view == ActiveView::DiffExplorer {
            return match key.code {
                KeyCode::Char('b') => {
                    if bracket == ']' {
                        Some(Action::NextBookmark)
                    } else {
                        Some(Action::PrevBookmark)
                    }
                }
                _ => {
                    if bracket == ']' {
                        Some(Action::JumpNextHunk)
                    } else {
                        Some(Action::JumpPrevHunk)
                    }
                }
            };
        }
    }

    // Priority 4.2: Pending mark / jump-to-mark (m + letter / ' + letter)
    if ctx.mark_pending {
        return match key.code {
            KeyCode::Char(c) if c.is_ascii_lowercase() => Some(Action::SetNamedBookmark(c)),
            _ => None,
        };
    }
    if ctx.jump_mark_pending {
        return match key.code {
            KeyCode::Char(c) if c.is_ascii_lowercase() => Some(Action::JumpToNamedBookmark(c)),
            _ => None,
        };
    }

    // Annotation navigation: Ctrl+]/[ for annotations.
    // Bare ] and [ enter bracket-pending state (handled in app.rs) so ]b / [b
    // can trigger bookmark navigation; the pending logic emits JumpNextHunk /
    // JumpPrevHunk on the next keypress if it's not 'b'.
    if ctx.active_view == ActiveView::DiffExplorer && key.modifiers.contains(KeyModifiers::CONTROL)
    {
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

    // Priority 5.5: Agent outputs — focus-based bindings
    if ctx.active_view == ActiveView::AgentOutputs {
        match ctx.focus {
            FocusPanel::AgentRunList => {
                if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Some(Action::KillAgentProcess);
                }
                return match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(Action::AgentOutputsUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Action::AgentOutputsDown),
                    KeyCode::Char('y') => Some(Action::AgentOutputsCopyPrompt),
                    KeyCode::Char('w') => Some(Action::AgentOutputsSwitchWorktree),
                    KeyCode::Char('o') => Some(Action::SwitchToAgentOutputs),
                    _ => None,
                };
            }
            FocusPanel::AgentOutput => {
                // PTY focus is handled at Priority 0 — all keys go to PTY.
                // This branch is only reached if pty_focus is false
                // (e.g. the agent has exited). Show scroll/navigation.
                return match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(Action::PtyScrollUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Action::PtyScrollDown),
                    KeyCode::Char('o') => Some(Action::SwitchToAgentOutputs),
                    _ => None,
                };
            }
            _ => {} // Other focus panels — fall through
        }
    }

    // Priority 5.6: Feedback summary view
    if ctx.active_view == ActiveView::FeedbackSummary {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::FeedbackSummaryUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::FeedbackSummaryDown),
            KeyCode::Char('y') => Some(Action::FeedbackSummaryCopyJson),
            KeyCode::Char('p') => Some(Action::FeedbackSummaryCopyPrompt),
            KeyCode::Esc | KeyCode::Char('F') => Some(Action::ToggleFeedbackSummary),
            _ => None,
        };
    }

    // Priority 6: Diff explorer global bindings
    if !review_composing {
        match key.code {
            KeyCode::Tab => return Some(Action::ToggleViewMode),
            KeyCode::Char('w') if !ctx.visual_mode_active => return Some(Action::ToggleWhitespace),

            KeyCode::Char('/') => {
                return match ctx.focus {
                    FocusPanel::Navigator => Some(Action::StartSearch),
                    FocusPanel::DiffView => Some(Action::StartDiffSearch),
                    _ => None,
                }
            }
            KeyCode::Char('s') if !ctx.visual_mode_active => return Some(Action::StageFile),
            KeyCode::Char('u') if !ctx.visual_mode_active => return Some(Action::UnstageFile),
            KeyCode::Char('r') if !ctx.visual_mode_active => return Some(Action::RestoreFile),
            KeyCode::Char('c') if !ctx.visual_mode_active => return Some(Action::OpenCommitDialog),
            KeyCode::Char('o') if !ctx.visual_mode_active => {
                return Some(Action::SwitchToAgentOutputs)
            }
            KeyCode::Char('F') => return Some(Action::ToggleFeedbackSummary),
            KeyCode::Char('S') if !ctx.visual_mode_active => {
                return Some(Action::ToggleStatsDashboard)
            }
            KeyCode::Char('R') => return Some(Action::RefreshDiff),
            KeyCode::Char('n') if !ctx.visual_mode_active => {
                return match ctx.focus {
                    FocusPanel::DiffView => Some(Action::DiffSearchNext),
                    FocusPanel::Navigator => Some(Action::NextUnreviewed),
                    _ => None,
                }
            }
            KeyCode::Char('t') if !ctx.visual_mode_active => return Some(Action::OpenTargetDialog),
            KeyCode::Char('T') if !ctx.visual_mode_active => return Some(Action::ToggleTreeView),
            KeyCode::Char('C') if !ctx.visual_mode_active => return Some(Action::ToggleChecklist),
            KeyCode::Char('?') => return Some(Action::ToggleWhichKey),
            KeyCode::Char(':') if !ctx.visual_mode_active => return Some(Action::OpenCommandBar),
            _ => {}
        }
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
            KeyCode::Char('1') => Some(Action::SetLineScore(1)),
            KeyCode::Char('2') => Some(Action::SetLineScore(2)),
            KeyCode::Char('3') => Some(Action::SetLineScore(3)),
            KeyCode::Char('4') => Some(Action::SetLineScore(4)),
            KeyCode::Char('5') => Some(Action::SetLineScore(5)),
            KeyCode::Char('0') => Some(Action::RemoveLineScore),
            _ => None,
        };
    }

    // Priority 8: Focus-dependent bindings
    match ctx.focus {
        FocusPanel::Navigator => {
            if ctx.tree_mode && ctx.tree_z_pending {
                return match key.code {
                    KeyCode::Char('M') => Some(Action::TreeCollapseAll),
                    KeyCode::Char('R') => Some(Action::TreeExpandAll),
                    _ => None,
                };
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(Action::NavigatorUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::NavigatorDown),
                KeyCode::Char('g') => Some(Action::NavigatorTop),
                KeyCode::Char('G') => Some(Action::NavigatorBottom),
                KeyCode::Char('m') => Some(Action::ToggleFileReviewed),
                KeyCode::Enter if ctx.tree_mode => Some(Action::TreeToggleCollapse),
                KeyCode::Enter => Some(Action::FocusDiffView),
                KeyCode::Char('x') if ctx.tree_mode => Some(Action::TreeToggleCollapse),
                KeyCode::Char('z') if ctx.tree_mode => {
                    // z prefix for fold commands; handled via z_pending state
                    None
                }
                _ => None,
            }
        }
        FocusPanel::DiffView => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
            KeyCode::Char('g') => Some(Action::ScrollToTop),
            KeyCode::Char('G') => Some(Action::ScrollToBottom),
            KeyCode::PageUp => Some(Action::ScrollPageUp),
            KeyCode::PageDown => Some(Action::ScrollPageDown),
            KeyCode::Char(' ') => Some(Action::ExpandContext),
            KeyCode::Char('e') => Some(Action::OpenInEditor),
            KeyCode::Char('v') | KeyCode::Char('V') => Some(Action::EnterVisualMode),
            KeyCode::Char('i') => Some(Action::OpenCommentEditor),
            KeyCode::Char('b') => Some(Action::ToggleBookmark),
            KeyCode::Char('B') => Some(Action::ToggleBookmarkList),
            KeyCode::Char('p') => Some(Action::TogglePromptPreview),
            KeyCode::Char('y') => Some(Action::CopyPromptToClipboard),
            KeyCode::Char('a') => Some(Action::OpenAnnotationMenu),
            KeyCode::Char('N') => Some(Action::DiffSearchPrev),
            KeyCode::Char('1') => Some(Action::SetLineScore(1)),
            KeyCode::Char('2') => Some(Action::SetLineScore(2)),
            KeyCode::Char('3') => Some(Action::SetLineScore(3)),
            KeyCode::Char('4') => Some(Action::SetLineScore(4)),
            KeyCode::Char('5') => Some(Action::SetLineScore(5)),
            KeyCode::Char('0') if ctx.timeline_active => Some(Action::TimelineSelectAll),
            KeyCode::Char('0') => Some(Action::RemoveLineScore),
            _ => None,
        },
        FocusPanel::ReviewPanel => {
            // Text input for composing review
            if ctx.agentic_review_composing {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return match key.code {
                        KeyCode::Char('a') => Some(Action::TextCursorHome),
                        KeyCode::Char('e') => Some(Action::TextCursorEnd),
                        KeyCode::Char('w') => Some(Action::TextDeleteWord),
                        _ => None,
                    };
                }
                match key.code {
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        Some(Action::AgenticReviewNewline)
                    }
                    KeyCode::Enter => Some(Action::AgenticReviewConfirm),
                    KeyCode::Backspace => Some(Action::AgenticReviewBackspace),
                    KeyCode::Left => Some(Action::TextCursorLeft),
                    KeyCode::Right => Some(Action::TextCursorRight),
                    KeyCode::Home => Some(Action::TextCursorHome),
                    KeyCode::End => Some(Action::TextCursorEnd),
                    KeyCode::Char(c) => Some(Action::AgenticReviewChar(c)),
                    _ => None,
                }
            } else {
                // Panel is open but not composing — scroll bindings
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(Action::AgenticReviewPanelUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Action::AgenticReviewPanelDown),
                    _ => None,
                }
            }
        }
        FocusPanel::ChecklistPanel => {
            // Delegate to existing checklist bindings
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(Action::ChecklistUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::ChecklistDown),
                KeyCode::Char(' ') | KeyCode::Enter => Some(Action::ChecklistToggleItem),
                KeyCode::Char('n') => Some(Action::ChecklistAddNote),
                _ => None,
            }
        }
        // AgentRunList and AgentOutput are handled at Priority 5.5 above
        FocusPanel::AgentRunList | FocusPanel::AgentOutput => None,
    }
}

/// Map a mouse event to an action based on current app context.
pub fn map_mouse_to_action(mouse: MouseEvent, ctx: &MouseContext<'_>) -> Option<Action> {
    match mouse.kind {
        // Scroll wheel
        MouseEventKind::ScrollUp => match ctx.panel_at(mouse.column, mouse.row) {
            Some(Panel::Navigator) => Some(Action::NavigatorUp),
            Some(Panel::DiffView) => Some(Action::ScrollUp),
            _ => None,
        },
        MouseEventKind::ScrollDown => match ctx.panel_at(mouse.column, mouse.row) {
            Some(Panel::Navigator) => Some(Action::NavigatorDown),
            Some(Panel::DiffView) => Some(Action::ScrollDown),
            _ => None,
        },
        // Left click
        MouseEventKind::Down(MouseButton::Left) => {
            match ctx.panel_at(mouse.column, mouse.row) {
                Some(Panel::Navigator) => {
                    let visible_index = ctx.navigator_row_to_visible_index(mouse.row);
                    visible_index
                        .and_then(|idx| ctx.navigator_visible_entries.get(idx))
                        .map(|(_, entry)| Action::SelectFile(entry.delta_index))
                }
                Some(Panel::DiffView) => {
                    // Click to focus diff view + position cursor
                    Some(Action::FocusDiffView)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::app_state::{ActiveView, CategoryPickerPhase, FocusPanel};

    fn create_global_search_context() -> KeyContext {
        KeyContext {
            focus: FocusPanel::DiffView,
            search_active: false,
            diff_search_active: false,
            global_search_active: true,
            commit_dialog_open: false,
            target_dialog_open: false,
            comment_editor_open: false,
            category_picker_open: false,
            category_picker_phase: CategoryPickerPhase::SelectCategory,
            agent_selector_open: false,
            annotation_menu_open: false,
            restore_confirm_open: false,
            settings_open: false,
            visual_mode_active: false,
            active_view: ActiveView::DiffExplorer,
            pty_focus: false,
            checklist_panel_open: false,
            bookmark_list_open: false,
            which_key_visible: false,
            tree_mode: false,
            tree_z_pending: false,
            bracket_pending: None,
            mark_pending: false,
            jump_mark_pending: false,
            command_bar_active: false,
            file_picker_active: false,
            agentic_review_modal_open: false,
            agentic_review_panel_open: false,
            agentic_review_composing: false,
            window_pending: false,
            stats_dashboard_open: false,
            timeline_active: false,
        }
    }

    fn create_agent_output_pty_context() -> KeyContext {
        KeyContext {
            focus: FocusPanel::AgentOutput,
            search_active: false,
            diff_search_active: false,
            global_search_active: false,
            commit_dialog_open: false,
            target_dialog_open: false,
            comment_editor_open: false,
            category_picker_open: false,
            category_picker_phase: CategoryPickerPhase::SelectCategory,
            agent_selector_open: false,
            annotation_menu_open: false,
            restore_confirm_open: false,
            settings_open: false,
            visual_mode_active: false,
            active_view: ActiveView::AgentOutputs,
            pty_focus: true,
            checklist_panel_open: false,
            bookmark_list_open: false,
            which_key_visible: false,
            tree_mode: false,
            tree_z_pending: false,
            bracket_pending: None,
            mark_pending: false,
            jump_mark_pending: false,
            command_bar_active: false,
            file_picker_active: false,
            agentic_review_modal_open: false,
            agentic_review_panel_open: false,
            agentic_review_composing: false,
            window_pending: false,
            stats_dashboard_open: false,
            timeline_active: false,
        }
    }

    fn create_review_composing_context() -> KeyContext {
        KeyContext {
            focus: FocusPanel::ReviewPanel,
            search_active: false,
            diff_search_active: false,
            global_search_active: false,
            commit_dialog_open: false,
            target_dialog_open: false,
            comment_editor_open: false,
            category_picker_open: false,
            category_picker_phase: CategoryPickerPhase::SelectCategory,
            agent_selector_open: false,
            annotation_menu_open: false,
            restore_confirm_open: false,
            settings_open: false,
            visual_mode_active: false,
            active_view: ActiveView::DiffExplorer,
            pty_focus: false,
            checklist_panel_open: false,
            bookmark_list_open: false,
            which_key_visible: false,
            tree_mode: false,
            tree_z_pending: false,
            bracket_pending: None,
            mark_pending: false,
            jump_mark_pending: false,
            command_bar_active: false,
            file_picker_active: false,
            agentic_review_modal_open: false,
            agentic_review_panel_open: true,
            agentic_review_composing: true,
            window_pending: false,
            stats_dashboard_open: false,
            timeline_active: false,
        }
    }

    #[test]
    fn test_global_search_n_key_inserts_character() {
        let ctx = create_global_search_context();
        let key_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let action = map_key_to_action(key_n, &ctx);

        match action {
            Some(Action::GlobalSearchChar('n')) => {
                // This is the expected behavior - 'n' should insert into search query
            }
            other => panic!("Expected GlobalSearchChar('n'), got {:?}", other),
        }
    }

    #[test]
    fn test_global_search_ctrl_n_navigates_next() {
        let ctx = create_global_search_context();
        let key_ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
        let action = map_key_to_action(key_ctrl_n, &ctx);

        match action {
            Some(Action::GlobalSearchNext) => {
                // This is the expected behavior - Ctrl+N should navigate to next match
            }
            other => panic!("Expected GlobalSearchNext, got {:?}", other),
        }
    }

    #[test]
    fn test_global_search_ctrl_p_navigates_prev() {
        let ctx = create_global_search_context();
        let key_ctrl_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        let action = map_key_to_action(key_ctrl_p, &ctx);

        match action {
            Some(Action::GlobalSearchPrev) => {
                // This is the expected behavior - Ctrl+P should navigate to previous match
            }
            other => panic!("Expected GlobalSearchPrev, got {:?}", other),
        }
    }

    #[test]
    fn test_global_search_enter_navigates_next() {
        let ctx = create_global_search_context();
        let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let action = map_key_to_action(key_enter, &ctx);

        match action {
            Some(Action::GlobalSearchNext) => {
                // This is the expected behavior - Enter should navigate to next match
            }
            other => panic!("Expected GlobalSearchNext, got {:?}", other),
        }
    }

    #[test]
    fn test_global_search_all_chars_insert() {
        let ctx = create_global_search_context();

        // Test various characters that should all insert into the search query
        let test_chars = ['f', 'u', 'n', 'c', 't', 'i', 'o', 'n'];

        for ch in test_chars {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            let action = map_key_to_action(key, &ctx);

            match action {
                Some(Action::GlobalSearchChar(c)) if c == ch => {
                    // Expected behavior - character should insert
                }
                other => panic!("Expected GlobalSearchChar('{}'), got {:?}", ch, other),
            }
        }
    }

    #[test]
    fn test_window_prefix_takes_priority_over_pty_focus() {
        let mut ctx = create_agent_output_pty_context();

        let ctrl_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert!(matches!(
            map_key_to_action(ctrl_w, &ctx),
            Some(Action::WindowPrefix)
        ));

        ctx.window_pending = true;
        let key_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(matches!(
            map_key_to_action(key_h, &ctx),
            Some(Action::FocusPaneLeft)
        ));
    }

    #[test]
    fn test_review_panel_text_input_overrides_global_bindings() {
        let ctx = create_review_composing_context();
        for c in ['s', 'u'] {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            assert!(matches!(
                map_key_to_action(key, &ctx),
                Some(Action::AgenticReviewChar(ch)) if ch == c
            ));
        }
    }
}
