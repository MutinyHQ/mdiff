use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::cell::Cell;
use std::path::PathBuf;
use std::time::Duration;

use crate::action::Action;
use crate::async_diff::{DiffRequest, DiffWorker};
use crate::components::action_hud::ActionHud;
use crate::components::commit_dialog::render_commit_dialog;
use crate::components::context_bar::ContextBar;
use crate::components::diff_view::DiffView;
use crate::components::navigator::Navigator;
use crate::components::worktree_browser::WorktreeBrowser;
use crate::components::Component;
use crate::event::{map_key_to_action, Event, EventReader};
use crate::git::commands::GitCli;
use crate::git::types::{ComparisonTarget, DiffLineOrigin, FileDelta};
use crate::git::worktree;
use crate::highlight::HighlightEngine;
use crate::state::app_state::{ActiveView, FocusPanel};
use crate::state::{AppState, DiffOptions, DiffViewMode};
use crate::tui::Tui;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub struct App {
    state: AppState,
    worker: DiffWorker,
    target: ComparisonTarget,
    generation: u64,
    highlight_engine: HighlightEngine,
    git_cli: GitCli,
    status_clear_countdown: u32,
    repo_path: PathBuf,
    nav_area: Cell<Rect>,
}

impl App {
    pub fn new(
        diff_options: DiffOptions,
        open_worktree_browser: bool,
        target: ComparisonTarget,
        repo_path: PathBuf,
    ) -> Self {
        let mut state = AppState::new(diff_options);
        state.target_label = match &target {
            ComparisonTarget::HeadVsWorkdir => "HEAD".to_string(),
            ComparisonTarget::Branch(name) => name.clone(),
            ComparisonTarget::Commit(oid) => format!("{:.7}", oid),
            ComparisonTarget::Ref(name) => name.clone(),
        };
        if open_worktree_browser {
            state.active_view = ActiveView::WorktreeBrowser;
        }
        let worker = DiffWorker::new(repo_path.clone());
        let highlight_engine = HighlightEngine::new();
        let git_cli = GitCli::new(&repo_path);
        Self {
            state,
            worker,
            target,
            generation: 0,
            highlight_engine,
            git_cli,
            status_clear_countdown: 0,
            repo_path,
            nav_area: Cell::new(Rect::default()),
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        self.request_diff();
        if self.state.active_view == ActiveView::WorktreeBrowser {
            self.refresh_worktrees();
        }

        let mut events = EventReader::new(Duration::from_millis(50));

        let context_bar = ContextBar;
        let navigator = Navigator;
        let diff_view = DiffView;
        let action_hud = ActionHud;
        let worktree_browser = WorktreeBrowser;

        loop {
            self.poll_diff_results();

            terminal.draw(|frame| {
                let outer = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Min(3),
                        Constraint::Length(1),
                    ])
                    .split(frame.area());

                context_bar.render(frame, outer[0], &self.state);

                match self.state.active_view {
                    ActiveView::DiffExplorer => {
                        let main = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
                            .split(outer[1]);

                        self.nav_area.set(main[0]);
                        navigator.render(frame, main[0], &self.state);
                        diff_view.render(frame, main[1], &self.state);
                    }
                    ActiveView::WorktreeBrowser => {
                        worktree_browser.render(frame, outer[1], &self.state);
                    }
                }

                action_hud.render(frame, outer[2], &self.state);

                // Render commit dialog overlay if open
                if self.state.commit_dialog_open {
                    render_commit_dialog(frame, &self.state);
                }
            })?;

            // Wait for at least one event, then drain all pending events
            // to avoid input lag from buffered scroll/key events.
            let first = events.next().await;
            let mut pending = Vec::new();
            if let Some(ev) = first {
                pending.push(ev);
            }
            while let Some(ev) = events.try_next() {
                pending.push(ev);
            }

            // Coalesce: collapse consecutive scroll actions into net movement
            let mut scroll_delta: i32 = 0;
            let mut actions: Vec<Action> = Vec::new();

            for event in pending {
                let action = match event {
                    Event::Key(key) => map_key_to_action(
                        key,
                        self.state.focus,
                        self.state.navigator.search_active,
                        self.state.commit_dialog_open,
                        self.state.active_view,
                    ),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(w, h) => Some(Action::Resize(w, h)),
                    Event::Tick => Some(Action::Tick),
                };
                if let Some(action) = action {
                    match action {
                        Action::ScrollUp => scroll_delta -= 1,
                        Action::ScrollDown => scroll_delta += 1,
                        other => actions.push(other),
                    }
                }
            }

            // Apply coalesced scroll
            if scroll_delta < 0 {
                for _ in 0..(-scroll_delta) {
                    self.update(Action::ScrollUp);
                }
            } else if scroll_delta > 0 {
                for _ in 0..scroll_delta {
                    self.update(Action::ScrollDown);
                }
            }

            // Apply remaining actions
            for action in actions {
                self.update(action);
            }

            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn request_diff(&mut self) {
        self.generation += 1;
        self.state.diff.loading = true;
        self.worker.request(DiffRequest {
            generation: self.generation,
            target: self.target.clone(),
            options: self.state.diff.options.clone(),
        });
    }

    fn poll_diff_results(&mut self) {
        while let Some(result) = self.worker.try_recv() {
            if result.generation < self.generation {
                continue;
            }
            self.state.diff.loading = false;
            match result.deltas {
                Ok(deltas) => {
                    self.state.navigator.update_from_deltas(&deltas);
                    self.state.diff.deltas = deltas;
                    if !self.state.diff.deltas.is_empty() && self.state.diff.selected_file.is_none()
                    {
                        self.state.diff.selected_file = Some(0);
                        self.update_highlights();
                    }
                }
                Err(_e) => {
                    self.state.diff.deltas.clear();
                    self.state.navigator.update_from_deltas(&[]);
                }
            }
        }
    }

    fn update_highlights(&mut self) {
        let Some(delta) = self.state.diff.selected_delta() else {
            self.state.diff.old_highlights.clear();
            self.state.diff.new_highlights.clear();
            return;
        };

        // Clone what we need to avoid borrow conflict
        let path = delta.path.clone();
        let (old_content, old_line_count) = reconstruct_content(delta, ContentSide::Old);
        let (new_content, new_line_count) = reconstruct_content(delta, ContentSide::New);

        self.state.diff.old_highlights = self
            .highlight_engine
            .highlight_lines(&path, &old_content)
            .unwrap_or_else(|| vec![Vec::new(); old_line_count + 1]);

        self.state.diff.new_highlights = self
            .highlight_engine
            .highlight_lines(&path, &new_content)
            .unwrap_or_else(|| vec![Vec::new(); new_line_count + 1]);
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.state.should_quit = true;
            }
            Action::NavigatorUp => {
                self.state.navigator.select_up();
                self.sync_selection();
            }
            Action::NavigatorDown => {
                self.state.navigator.select_down();
                self.sync_selection();
            }
            Action::SelectFile(idx) => {
                self.state.diff.selected_file = Some(idx);
                self.state.diff.scroll_offset = 0;
                // Sync navigator selection to match clicked file
                if let Some(vis_idx) = self
                    .state
                    .navigator
                    .visible_entries()
                    .iter()
                    .position(|(_, e)| e.delta_index == idx)
                {
                    self.state.navigator.selected = vis_idx;
                }
                self.state.focus = FocusPanel::Navigator;
                self.update_highlights();
            }
            Action::ScrollUp => {
                self.state.diff.scroll_offset = self.state.diff.scroll_offset.saturating_sub(1);
            }
            Action::ScrollDown => {
                let max = self.state.diff.total_lines();
                if self.state.diff.scroll_offset < max {
                    self.state.diff.scroll_offset += 1;
                }
            }
            Action::ScrollPageUp => {
                self.state.diff.scroll_offset = self.state.diff.scroll_offset.saturating_sub(20);
            }
            Action::ScrollPageDown => {
                let max = self.state.diff.total_lines();
                self.state.diff.scroll_offset = (self.state.diff.scroll_offset + 20).min(max);
            }
            Action::ToggleViewMode => {
                self.state.diff.options.view_mode = match self.state.diff.options.view_mode {
                    DiffViewMode::Split => DiffViewMode::Unified,
                    DiffViewMode::Unified => DiffViewMode::Split,
                };
            }
            Action::ToggleWhitespace => {
                self.state.diff.options.ignore_whitespace =
                    !self.state.diff.options.ignore_whitespace;
                self.request_diff();
            }
            Action::FocusNavigator => {
                self.state.focus = FocusPanel::Navigator;
            }
            Action::FocusDiffView => {
                self.state.focus = FocusPanel::DiffView;
            }
            Action::StartSearch => {
                self.state.navigator.start_search();
                self.state.focus = FocusPanel::Navigator;
            }
            Action::EndSearch => {
                self.state.navigator.end_search();
                self.sync_selection();
            }
            Action::SearchChar(c) => {
                self.state.navigator.search_push(c);
                self.sync_selection();
            }
            Action::SearchBackspace => {
                self.state.navigator.search_pop();
                self.sync_selection();
            }
            Action::ToggleWorktreeBrowser => {
                self.state.active_view = match self.state.active_view {
                    ActiveView::DiffExplorer => {
                        self.refresh_worktrees();
                        ActiveView::WorktreeBrowser
                    }
                    ActiveView::WorktreeBrowser => ActiveView::DiffExplorer,
                };
            }
            Action::WorktreeUp => {
                self.state.worktree.select_up();
            }
            Action::WorktreeDown => {
                self.state.worktree.select_down();
            }
            Action::WorktreeSelect => {
                if let Some(wt) = self.state.worktree.selected_worktree().cloned() {
                    let new_path = wt.path.clone();
                    self.repo_path = new_path.clone();
                    self.worker = DiffWorker::new(new_path.clone());
                    self.git_cli = GitCli::new(&new_path);
                    self.generation = 0;
                    self.state.diff.deltas.clear();
                    self.state.diff.selected_file = None;
                    self.state.diff.scroll_offset = 0;
                    self.state.navigator.entries.clear();
                    self.state.navigator.filtered_indices.clear();
                    self.state.active_view = ActiveView::DiffExplorer;
                    self.request_diff();
                    self.set_status(format!("Switched to: {}", wt.name), false);
                }
            }
            Action::WorktreeRefresh => {
                self.refresh_worktrees();
            }
            Action::WorktreeFreeze => {
                if let Some(wt) = self.state.worktree.selected_worktree().cloned() {
                    let freeze_cli = GitCli::new(&wt.path);
                    match freeze_cli
                        .stage_all()
                        .and_then(|()| freeze_cli.commit("Agent Checkpoint"))
                    {
                        Ok(()) => {
                            self.set_status(format!("Frozen: {}", wt.name), false);
                            self.refresh_worktrees();
                        }
                        Err(e) => {
                            self.set_status(format!("Freeze failed: {e}"), true);
                        }
                    }
                }
            }
            Action::WorktreeBack => {
                self.state.active_view = ActiveView::DiffExplorer;
            }
            Action::StageFile => {
                if let Some(path) = self.selected_file_path() {
                    match self.git_cli.stage_file(&path) {
                        Ok(()) => {
                            self.set_status(format!("Staged: {}", path.display()), false);
                            self.request_diff();
                        }
                        Err(e) => {
                            self.set_status(format!("Stage failed: {e}"), true);
                        }
                    }
                }
            }
            Action::UnstageFile => {
                if let Some(path) = self.selected_file_path() {
                    match self.git_cli.unstage_file(&path) {
                        Ok(()) => {
                            self.set_status(format!("Unstaged: {}", path.display()), false);
                            self.request_diff();
                        }
                        Err(e) => {
                            self.set_status(format!("Unstage failed: {e}"), true);
                        }
                    }
                }
            }
            Action::RestoreFile => {
                if let Some(path) = self.selected_file_path() {
                    match self.git_cli.restore_file(&path) {
                        Ok(()) => {
                            self.set_status(format!("Restored: {}", path.display()), false);
                            self.request_diff();
                        }
                        Err(e) => {
                            self.set_status(format!("Restore failed: {e}"), true);
                        }
                    }
                }
            }
            Action::OpenCommitDialog => {
                self.state.commit_dialog_open = true;
                self.state.commit_message.clear();
            }
            Action::CancelCommit => {
                self.state.commit_dialog_open = false;
                self.state.commit_message.clear();
            }
            Action::ConfirmCommit => {
                if self.state.commit_message.trim().is_empty() {
                    self.set_status("Commit message cannot be empty".to_string(), true);
                } else {
                    let msg = self.state.commit_message.clone();
                    match self.git_cli.commit(&msg) {
                        Ok(()) => {
                            self.set_status("Committed successfully".to_string(), false);
                            self.state.commit_dialog_open = false;
                            self.state.commit_message.clear();
                            self.request_diff();
                        }
                        Err(e) => {
                            self.set_status(format!("Commit failed: {e}"), true);
                        }
                    }
                }
            }
            Action::CommitChar(c) => {
                self.state.commit_message.push(c);
            }
            Action::CommitBackspace => {
                self.state.commit_message.pop();
            }
            Action::Tick => {
                if self.status_clear_countdown > 0 {
                    self.status_clear_countdown -= 1;
                    if self.status_clear_countdown == 0 {
                        self.state.status_message = None;
                    }
                }
            }
            Action::Resize(_, _) => {}
            _ => {}
        }
    }

    fn handle_mouse(&self, mouse: MouseEvent) -> Option<Action> {
        match mouse.kind {
            MouseEventKind::ScrollUp => Some(Action::ScrollUp),
            MouseEventKind::ScrollDown => Some(Action::ScrollDown),
            MouseEventKind::Down(MouseButton::Left) => {
                if self.state.active_view != ActiveView::DiffExplorer {
                    return None;
                }
                let nav = self.nav_area.get();
                let col = mouse.column;
                let row = mouse.row;

                // Check if click is inside the navigator area (excluding border)
                if col > nav.x
                    && col < nav.x + nav.width.saturating_sub(1)
                    && row > nav.y
                    && row < nav.y + nav.height.saturating_sub(1)
                {
                    let inner_height = nav.height.saturating_sub(2) as usize;
                    let selected = self.state.navigator.selected;
                    let scroll = if selected >= inner_height {
                        selected - inner_height + 1
                    } else {
                        0
                    };

                    let clicked_row = (row - nav.y - 1) as usize;
                    let visible_idx = scroll + clicked_row;
                    let visible = self.state.navigator.visible_entries();
                    if visible_idx < visible.len() {
                        let (_entry_idx, entry) = &visible[visible_idx];
                        return Some(Action::SelectFile(entry.delta_index));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn refresh_worktrees(&mut self) {
        match worktree::list_worktrees(&self.repo_path) {
            Ok(wts) => {
                self.state.worktree.worktrees = wts;
                self.state.worktree.loading = false;
            }
            Err(e) => {
                self.set_status(format!("Failed to list worktrees: {e}"), true);
            }
        }
    }

    fn set_status(&mut self, msg: String, is_error: bool) {
        self.state.status_message = Some((msg, is_error));
        // ~3 seconds at 50ms tick rate
        self.status_clear_countdown = 60;
    }

    fn selected_file_path(&self) -> Option<PathBuf> {
        self.state
            .diff
            .selected_file
            .and_then(|idx| self.state.diff.deltas.get(idx))
            .map(|delta| delta.path.clone())
    }

    fn sync_selection(&mut self) {
        if let Some(delta_idx) = self.state.navigator.selected_delta_index() {
            let changed = self.state.diff.selected_file != Some(delta_idx);
            self.state.diff.selected_file = Some(delta_idx);
            self.state.diff.scroll_offset = 0;
            if changed {
                self.update_highlights();
            }
        }
    }
}

enum ContentSide {
    Old,
    New,
}

/// Reconstruct file content from diff hunks for one side.
/// Returns (content_string, max_line_number).
/// Lines are indexed by their original line numbers, with gaps filled by empty lines.
fn reconstruct_content(delta: &FileDelta, side: ContentSide) -> (String, usize) {
    let mut lines: Vec<(u32, String)> = Vec::new();

    for hunk in &delta.hunks {
        for line in &hunk.lines {
            match (&side, &line.origin) {
                (ContentSide::Old, DiffLineOrigin::Context | DiffLineOrigin::Deletion) => {
                    if let Some(n) = line.old_lineno {
                        lines.push((n, line.content.trim_end_matches('\n').to_string()));
                    }
                }
                (ContentSide::New, DiffLineOrigin::Context | DiffLineOrigin::Addition) => {
                    if let Some(n) = line.new_lineno {
                        lines.push((n, line.content.trim_end_matches('\n').to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    if lines.is_empty() {
        return (String::new(), 0);
    }

    let max_line = lines.iter().map(|(n, _)| *n).max().unwrap_or(0) as usize;

    // Build content indexed by line number (sparse → dense)
    let mut content_lines = vec![String::new(); max_line + 1];
    for (n, text) in &lines {
        content_lines[*n as usize] = text.clone();
    }

    let content = content_lines.join("\n");
    (content, max_line)
}

pub fn parse_target(target: Option<&str>) -> ComparisonTarget {
    match target {
        None => ComparisonTarget::HeadVsWorkdir,
        Some(s) => {
            if s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(oid) = git2::Oid::from_str(s) {
                    return ComparisonTarget::Commit(oid);
                }
            }
            ComparisonTarget::Branch(s.to_string())
        }
    }
}
