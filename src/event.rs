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
    pub which_key_visible: bool,
    pub tree_mode: bool,
    pub tree_z_pending: bool,
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
    // Priority 0: PTY focus mode - forward ALL keys except Esc to the PTY.
    // This must come first so Ctrl+C/D go to the agent, not quit mdiff.
    if ctx.pty_focus {
        return match key.code {
            KeyCode::Esc => Some(Action::ExitPtyFocus),
            _ => Some(Action::PtyInput(key)),
        };
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

    // Priority 4: Global bindings (always active)
    match key.code {
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::ToggleWorktreeBrowser)
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
        _ => {}
    }

    // Annotation navigation moved to Ctrl modifier, hunk nav on bare keys
    if ctx.active_view == ActiveView::DiffExplorer {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char(']') => return Some(Action::NextAnnotation),
                KeyCode::Char('[') => return Some(Action::PrevAnnotation),
                _ => {}
            }
        }
        match key.code {
            KeyCode::Char(']') => return Some(Action::JumpNextHunk),
            KeyCode::Char('[') => return Some(Action::JumpPrevHunk),
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
            KeyCode::Char('y') => Some(Action::AgentOutputsCopyPrompt),
            KeyCode::Char('w') => Some(Action::AgentOutputsSwitchWorktree),
            KeyCode::Enter => Some(Action::EnterPtyFocus),
            KeyCode::Esc => Some(Action::SwitchToAgentOutputs), // toggle back
            _ => None,
        };
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
    match key.code {
        KeyCode::Tab => return Some(Action::ToggleViewMode),
        KeyCode::Char('w') if !ctx.visual_mode_active => return Some(Action::ToggleWhitespace),

        KeyCode::Char('/') => {
            return match ctx.focus {
                FocusPanel::Navigator => Some(Action::StartSearch),
                FocusPanel::DiffView => Some(Action::StartDiffSearch),
            }
        }
        KeyCode::Char('s') if !ctx.visual_mode_active => return Some(Action::StageFile),
        KeyCode::Char('u') if !ctx.visual_mode_active => return Some(Action::UnstageFile),
        KeyCode::Char('r') if !ctx.visual_mode_active => return Some(Action::RestoreFile),
        KeyCode::Char('c') if !ctx.visual_mode_active => return Some(Action::OpenCommitDialog),
        KeyCode::Char('o') if !ctx.visual_mode_active => return Some(Action::SwitchToAgentOutputs),
        KeyCode::Char('F') => return Some(Action::ToggleFeedbackSummary),
        KeyCode::Char('R') => return Some(Action::RefreshDiff),
        KeyCode::Char('n') if !ctx.visual_mode_active => {
            return match ctx.focus {
                FocusPanel::DiffView => Some(Action::DiffSearchNext),
                FocusPanel::Navigator => Some(Action::NextUnreviewed),
            }
        }
        KeyCode::Char('t') if !ctx.visual_mode_active => return Some(Action::OpenTargetDialog),
        KeyCode::Char('T') if !ctx.visual_mode_active => return Some(Action::ToggleTreeView),
        KeyCode::Char('C') if !ctx.visual_mode_active => return Some(Action::ToggleChecklist),
        KeyCode::Char('?') => return Some(Action::ToggleWhichKey),
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
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                    if ctx.tree_mode {
                        Some(Action::TreeToggleCollapse)
                    } else {
                        Some(Action::FocusDiffView)
                    }
                }
                KeyCode::Char('h') if ctx.tree_mode => Some(Action::TreeToggleCollapse),
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
            KeyCode::Left | KeyCode::Char('h') => Some(Action::FocusNavigator),
            KeyCode::PageUp => Some(Action::ScrollPageUp),
            KeyCode::PageDown => Some(Action::ScrollPageDown),
            KeyCode::Char(' ') => Some(Action::ExpandContext),
            KeyCode::Char('e') => Some(Action::OpenInEditor),
            KeyCode::Char('v') | KeyCode::Char('V') => Some(Action::EnterVisualMode),
            KeyCode::Char('i') => Some(Action::OpenCommentEditor),
            KeyCode::Char('p') => Some(Action::TogglePromptPreview),
            KeyCode::Char('y') => Some(Action::CopyPromptToClipboard),
            KeyCode::Char('a') => Some(Action::OpenAnnotationMenu),
            KeyCode::Char('N') => Some(Action::DiffSearchPrev),
            KeyCode::Char('1') => Some(Action::SetLineScore(1)),
            KeyCode::Char('2') => Some(Action::SetLineScore(2)),
            KeyCode::Char('3') => Some(Action::SetLineScore(3)),
            KeyCode::Char('4') => Some(Action::SetLineScore(4)),
            KeyCode::Char('5') => Some(Action::SetLineScore(5)),
            KeyCode::Char('0') => Some(Action::RemoveLineScore),
            _ => None,
        },
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
            which_key_visible: false,
            tree_mode: false,
            tree_z_pending: false,
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
}
