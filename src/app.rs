use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::cell::Cell;
use std::path::PathBuf;
use std::time::Duration;

use crate::action::Action;
use crate::async_diff::{DiffRequest, DiffWorker};
use crate::components::action_hud::{hud_height, ActionHud};
use crate::components::agent_outputs::AgentOutputs;
use crate::components::agent_selector::render_agent_selector;
use crate::components::annotation_menu::render_annotation_menu;
use crate::components::comment_editor::render_comment_editor;
use crate::components::commit_dialog::render_commit_dialog;
use crate::components::context_bar::ContextBar;
use crate::components::diff_view::DiffView;
use crate::components::navigator::Navigator;
use crate::components::prompt_preview::render_prompt_preview;
use crate::components::restore_confirm::render_restore_confirm;
use crate::components::settings_modal::render_settings_modal;
use crate::components::target_dialog::render_target_dialog;
use crate::components::worktree_browser::WorktreeBrowser;
use crate::components::Component;
use crate::config::{self, MdiffConfig, PersistentSettings};
use crate::context;
use crate::display_map::{build_display_map, DisplayRowInfo};
use crate::event::{map_key_to_action, Event, EventReader, KeyContext};
use crate::git::commands::GitCli;
use crate::git::types::{ComparisonTarget, DiffLineOrigin, FileDelta};
use crate::git::worktree;
use crate::highlight::HighlightEngine;
use crate::pty_runner::{key_event_to_bytes, PtyEvent, PtyRunner};
use crate::session;
use crate::state::agent_state::{AgentRun, AgentRunStatus};
use crate::state::annotation_state::{Annotation, LineAnchor};
use crate::state::app_state::{ActiveView, FocusPanel};
use crate::state::review_state::compute_diff_hashes;
use crate::state::settings_state::SETTINGS_ROW_COUNT;
use crate::state::{AppState, DiffOptions, DiffViewMode};
use crate::template;
use crate::theme::{next_theme, prev_theme, Theme};
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
    hud_collapse_countdown: u32,
    repo_path: PathBuf,
    nav_area: Cell<Rect>,
    config: MdiffConfig,
    pty_runner: Option<PtyRunner>,
}

impl App {
    pub fn new(
        diff_options: DiffOptions,
        open_worktree_browser: bool,
        target: ComparisonTarget,
        repo_path: PathBuf,
        config: MdiffConfig,
        context_lines: Option<usize>,
    ) -> Self {
        let theme = config.theme.clone();
        let mut state = AppState::new(diff_options, theme);
        state.target_label = match &target {
            ComparisonTarget::HeadVsWorkdir => "HEAD".to_string(),
            ComparisonTarget::Branch(name) => name.clone(),
            ComparisonTarget::Commit(oid) => format!("{:.7}", oid),
        };
        if open_worktree_browser {
            state.active_view = ActiveView::WorktreeBrowser;
        }
        if let Some(ctx) = context_lines {
            state.diff.display_context = ctx;
        }

        // Load session annotations
        state.annotations = session::load_session(&repo_path, &state.target_label);

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
            hud_collapse_countdown: 0,
            repo_path,
            nav_area: Cell::new(Rect::default()),
            config,
            pty_runner: None,
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
        let agent_outputs = AgentOutputs;

        loop {
            self.poll_diff_results();
            self.poll_pty_output();

            // Update viewport height for cursor auto-scroll calculations
            let term_size = terminal.size()?;
            let mut vh = term_size.height.saturating_sub(4) as usize; // context_bar + hud + borders
            if self.state.prompt_preview_visible {
                vh = vh * 60 / 100;
            }
            self.state.diff.viewport_height = vh;

            terminal.draw(|frame| {
                let hud_h = hud_height(&self.state, frame.area().width);
                let outer = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Min(3),
                        Constraint::Length(hud_h),
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

                        if self.state.prompt_preview_visible {
                            let vsplit = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([
                                    Constraint::Percentage(60),
                                    Constraint::Percentage(40),
                                ])
                                .split(main[1]);

                            diff_view.render(frame, vsplit[0], &self.state);
                            render_prompt_preview(frame, vsplit[1], &self.state);
                        } else {
                            diff_view.render(frame, main[1], &self.state);
                        }
                    }
                    ActiveView::WorktreeBrowser => {
                        worktree_browser.render(frame, outer[1], &self.state);
                    }
                    ActiveView::AgentOutputs => {
                        agent_outputs.render(frame, outer[1], &self.state);
                    }
                }

                action_hud.render(frame, outer[2], &self.state);

                // Render modal overlays (in priority order)
                if self.state.target_dialog_open {
                    render_target_dialog(frame, &self.state);
                }
                if self.state.commit_dialog_open {
                    render_commit_dialog(frame, &self.state);
                }
                if self.state.comment_editor_open {
                    render_comment_editor(frame, &self.state);
                }
                if self.state.annotation_menu_open {
                    render_annotation_menu(frame, &self.state);
                }
                if self.state.agent_selector.open {
                    render_agent_selector(frame, &self.state.agent_selector);
                }
                if self.state.restore_confirm_open {
                    render_restore_confirm(frame, &self.state);
                }
                if self.state.settings.open {
                    render_settings_modal(frame, &self.state);
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
                let ctx = KeyContext {
                    focus: self.state.focus,
                    search_active: self.state.navigator.search_active,
                    commit_dialog_open: self.state.commit_dialog_open,
                    target_dialog_open: self.state.target_dialog_open,
                    comment_editor_open: self.state.comment_editor_open,
                    agent_selector_open: self.state.agent_selector.open,
                    annotation_menu_open: self.state.annotation_menu_open,
                    restore_confirm_open: self.state.restore_confirm_open,
                    settings_open: self.state.settings.open,
                    visual_mode_active: self.state.selection.active,
                    active_view: self.state.active_view,
                    pty_focus: self.state.pty_focus,
                };
                let action = match event {
                    Event::Key(key) => map_key_to_action(key, &ctx),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize => Some(Action::Resize),
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

        // Save session on quit
        session::save_session(
            &self.repo_path,
            &self.state.target_label,
            &self.state.annotations,
        );

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
                    let new_hashes = compute_diff_hashes(&deltas);
                    self.state.review.on_diff_refresh(new_hashes);
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

    fn poll_pty_output(&mut self) {
        let Some(runner) = self.pty_runner.as_mut() else {
            return;
        };

        // Collect PTY output events
        let mut events = Vec::new();
        while let Some(event) = runner.try_recv() {
            events.push(event);
        }

        // Check if the child process has exited
        let exit_code = runner.try_wait();

        for event in events {
            match event {
                PtyEvent::Output(run_id, bytes) => {
                    if let Some(run) = self
                        .state
                        .agent_outputs
                        .runs
                        .iter_mut()
                        .find(|r| r.id == run_id)
                    {
                        run.terminal.process(&bytes);
                    }
                    // Auto-scroll to bottom when viewing the active run
                    let selected_id = self.state.agent_outputs.selected().map(|r| r.id);
                    if selected_id == Some(run_id) {
                        // Reset detail_scroll to 0 = "follow mode" (bottom)
                        self.state.agent_outputs.detail_scroll = 0;
                    }
                }
            }
        }

        // Also check if child exited (may not have sent Done event via reader)
        if let Some(code) = exit_code {
            // Find the running agent run and mark it done
            if let Some(run) = self
                .state
                .agent_outputs
                .runs
                .iter_mut()
                .find(|r| matches!(r.status, AgentRunStatus::Running))
            {
                run.status = if code == 0 {
                    AgentRunStatus::Success { exit_code: code }
                } else {
                    AgentRunStatus::Failed { exit_code: code }
                };
            }
            self.state.pty_focus = false;
            self.pty_runner = None;
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

        let syntax = &self.state.theme.syntax;
        self.state.diff.old_highlights = self
            .highlight_engine
            .highlight_lines(&path, &old_content, syntax)
            .unwrap_or_else(|| vec![Vec::new(); old_line_count + 1]);

        self.state.diff.new_highlights = self
            .highlight_engine
            .highlight_lines(&path, &new_content, syntax)
            .unwrap_or_else(|| vec![Vec::new(); new_line_count + 1]);
    }

    /// Build the display map for the currently selected file.
    fn current_display_map(&self) -> Vec<DisplayRowInfo> {
        let Some(delta) = self.state.diff.selected_delta() else {
            return Vec::new();
        };
        build_display_map(
            delta,
            self.state.diff.options.view_mode,
            self.state.diff.display_context,
            &self.state.diff.gap_expansions,
        )
    }

    /// Convert the current visual selection to a LineAnchor using the display map.
    fn selection_to_anchor(&self) -> Option<LineAnchor> {
        let delta = self.state.diff.selected_delta()?;
        let display_map = self.current_display_map();
        let (start, end) = self.state.selection.range();

        let file_path = delta.path.to_string_lossy().to_string();
        let mut min_line: Option<u32> = None;
        let mut max_line: Option<u32> = None;

        for row_idx in start..=end {
            if let Some(info) = display_map.get(row_idx) {
                for lineno in [info.old_lineno, info.new_lineno].iter().flatten() {
                    min_line = Some(min_line.map_or(*lineno, |m: u32| m.min(*lineno)));
                    max_line = Some(max_line.map_or(*lineno, |m: u32| m.max(*lineno)));
                }
            }
        }

        Some(LineAnchor {
            file_path,
            line_start: min_line.unwrap_or(1),
            line_end: max_line.unwrap_or(1),
        })
    }

    /// Convert the cursor row to a single-line LineAnchor (used when no visual selection is active).
    fn cursor_to_anchor(&self) -> Option<LineAnchor> {
        let delta = self.state.diff.selected_delta()?;
        let display_map = self.current_display_map();
        let info = display_map.get(self.state.diff.cursor_row)?;

        let file_path = delta.path.to_string_lossy().to_string();
        let lineno = info.new_lineno.or(info.old_lineno)?;
        Some(LineAnchor {
            file_path,
            line_start: lineno,
            line_end: lineno,
        })
    }

    /// Get a LineAnchor from either the visual selection or the cursor position.
    fn current_anchor(&self) -> Option<LineAnchor> {
        if self.state.selection.active {
            self.selection_to_anchor()
        } else {
            self.cursor_to_anchor()
        }
    }

    /// Render the prompt template for the current selection and optional comment.
    fn render_prompt_for_selection(&self, comment: &str) -> Option<String> {
        let delta = self.state.diff.selected_delta()?;
        let anchor = self.current_anchor()?;
        let ctx = context::extract_context(delta, &anchor, comment, self.config.context_padding);
        Some(template::render_template(
            &self.config.prompt_template,
            &ctx,
        ))
    }

    fn update(&mut self, action: Action) {
        // Auto-collapse HUD on first real command after expanding
        if self.state.hud_expanded {
            match action {
                Action::Tick
                | Action::Resize
                | Action::ToggleHud
                | Action::OpenSettings
                | Action::CloseSettings
                | Action::SettingsUp
                | Action::SettingsDown
                | Action::SettingsLeft
                | Action::SettingsRight => {}
                _ => {
                    self.state.hud_expanded = false;
                    self.hud_collapse_countdown = 0;
                }
            }
        }

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
            Action::NavigatorTop => {
                self.state.navigator.selected = 0;
                self.sync_selection();
            }
            Action::NavigatorBottom => {
                let len = self.state.navigator.visible_entries().len();
                if len > 0 {
                    self.state.navigator.selected = len - 1;
                }
                self.sync_selection();
            }
            Action::SelectFile(idx) => {
                self.state.diff.selected_file = Some(idx);
                self.state.diff.scroll_offset = 0;
                self.state.diff.cursor_row = 0;
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
                self.state.diff.cursor_row = self.state.diff.cursor_row.saturating_sub(1);
                // Auto-scroll if cursor goes above viewport
                if self.state.diff.cursor_row < self.state.diff.scroll_offset {
                    self.state.diff.scroll_offset = self.state.diff.cursor_row;
                }
            }
            Action::ScrollDown => {
                let max = self.current_display_map().len().saturating_sub(1);
                if self.state.diff.cursor_row < max {
                    self.state.diff.cursor_row += 1;
                }
                // Auto-scroll if cursor goes below viewport
                let vh = self.state.diff.viewport_height;
                if self.state.diff.cursor_row >= self.state.diff.scroll_offset + vh {
                    self.state.diff.scroll_offset = self.state.diff.cursor_row - vh + 1;
                }
                self.check_auto_review();
            }
            Action::ScrollToTop => {
                self.state.diff.cursor_row = 0;
                self.state.diff.scroll_offset = 0;
            }
            Action::ScrollToBottom => {
                let max = self.current_display_map().len().saturating_sub(1);
                self.state.diff.cursor_row = max;
                let vh = self.state.diff.viewport_height;
                self.state.diff.scroll_offset = max.saturating_sub(vh.saturating_sub(1));
                self.check_auto_review();
            }
            Action::ScrollPageUp => {
                let vh = self.state.diff.viewport_height;
                self.state.diff.cursor_row = self.state.diff.cursor_row.saturating_sub(vh);
                self.state.diff.scroll_offset = self.state.diff.scroll_offset.saturating_sub(vh);
            }
            Action::ScrollPageDown => {
                let vh = self.state.diff.viewport_height;
                let max = self.current_display_map().len().saturating_sub(1);
                self.state.diff.cursor_row = (self.state.diff.cursor_row + vh).min(max);
                if self.state.diff.cursor_row >= self.state.diff.scroll_offset + vh {
                    self.state.diff.scroll_offset = self.state.diff.cursor_row - vh + 1;
                }
                self.check_auto_review();
            }
            Action::ToggleViewMode => {
                self.state.diff.options.view_mode = match self.state.diff.options.view_mode {
                    DiffViewMode::Split => DiffViewMode::Unified,
                    DiffViewMode::Unified => DiffViewMode::Split,
                };
                // Exit visual mode on view mode change since display map changes
                self.state.selection.active = false;
                self.state.diff.cursor_row = 0;
                self.state.diff.scroll_offset = 0;
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
                // Ensure cursor is within visible viewport
                let vh = self.state.diff.viewport_height;
                let scroll = self.state.diff.scroll_offset;
                if self.state.diff.cursor_row < scroll || self.state.diff.cursor_row >= scroll + vh
                {
                    self.state.diff.cursor_row = scroll;
                }
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
                    ActiveView::DiffExplorer | ActiveView::AgentOutputs => {
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
                    self.state.review.reset();
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
                if self.selected_file_path().is_some() {
                    self.state.restore_confirm_open = true;
                }
            }
            Action::ConfirmRestore => {
                self.state.restore_confirm_open = false;
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
            Action::CancelRestore => {
                self.state.restore_confirm_open = false;
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
            Action::CommitNewline => {
                self.state.commit_message.push('\n');
            }

            // Target dialog
            Action::OpenTargetDialog => {
                self.state.target_dialog_open = true;
                self.state.target_dialog_input.clear();
            }
            Action::CancelTarget => {
                self.state.target_dialog_open = false;
                self.state.target_dialog_input.clear();
            }
            Action::TargetChar(c) => {
                self.state.target_dialog_input.push(c);
            }
            Action::TargetBackspace => {
                self.state.target_dialog_input.pop();
            }
            Action::ConfirmTarget => {
                let input = self.state.target_dialog_input.trim().to_string();
                if input.is_empty() {
                    // Reset to HEAD vs workdir
                    self.state.target_dialog_open = false;
                    self.state.target_dialog_input.clear();
                    self.apply_new_target(ComparisonTarget::HeadVsWorkdir, "HEAD".to_string());
                } else {
                    match self.validate_ref(&input) {
                        Ok((target, label)) => {
                            self.state.target_dialog_open = false;
                            self.state.target_dialog_input.clear();
                            self.apply_new_target(target, label);
                        }
                        Err(e) => {
                            self.set_status(format!("Invalid ref '{}': {}", input, e), true);
                        }
                    }
                }
            }

            // Visual selection
            Action::EnterVisualMode => {
                self.state.selection.active = true;
                self.state.selection.anchor = self.state.diff.cursor_row;
                self.state.selection.cursor = self.state.diff.cursor_row;
                self.state.focus = FocusPanel::DiffView;
            }
            Action::ExitVisualMode => {
                self.state.selection.active = false;
            }
            Action::ExtendSelectionUp => {
                self.state.selection.cursor = self.state.selection.cursor.saturating_sub(1);
                // Auto-scroll if cursor goes above viewport
                if self.state.selection.cursor < self.state.diff.scroll_offset {
                    self.state.diff.scroll_offset = self.state.selection.cursor;
                }
            }
            Action::ExtendSelectionDown => {
                let display_map = self.current_display_map();
                let max = display_map.len().saturating_sub(1);
                if self.state.selection.cursor < max {
                    self.state.selection.cursor += 1;
                }
                // Auto-scroll if cursor goes below viewport (approximation)
                // We don't have the exact viewport height here, so use a reasonable default
            }

            // Comment editor
            Action::OpenCommentEditor => {
                if !self.state.selection.active {
                    // Set a single-line selection at the cursor
                    self.state.selection.active = true;
                    self.state.selection.anchor = self.state.diff.cursor_row;
                    self.state.selection.cursor = self.state.diff.cursor_row;
                }
                self.state.comment_editor_open = true;
                self.state.comment_editor_text.clear();
            }
            Action::CancelComment => {
                self.state.comment_editor_open = false;
                self.state.comment_editor_text.clear();
                self.state.editing_annotation = None;
            }
            Action::ConfirmComment => {
                if !self.state.comment_editor_text.trim().is_empty() {
                    if let Some(editing) = self.state.editing_annotation.take() {
                        // Editing an existing annotation from the annotation menu
                        self.state.annotations.update_comment(
                            &editing.file_path,
                            editing.line_start,
                            editing.line_end,
                            &editing.old_comment,
                            &self.state.comment_editor_text,
                        );
                        self.set_status("Comment updated".to_string(), false);
                    } else if let Some(anchor) = self.selection_to_anchor() {
                        // Creating a new annotation from visual mode
                        let now = chrono::Utc::now().to_rfc3339();
                        self.state.annotations.add(Annotation {
                            anchor,
                            comment: self.state.comment_editor_text.clone(),
                            created_at: now,
                        });
                        self.set_status("Comment added".to_string(), false);
                    }
                }
                self.state.comment_editor_open = false;
                self.state.comment_editor_text.clear();
                self.state.selection.active = false;
                self.state.editing_annotation = None;
            }
            Action::CommentChar(c) => {
                self.state.comment_editor_text.push(c);
            }
            Action::CommentBackspace => {
                self.state.comment_editor_text.pop();
            }
            Action::CommentNewline => {
                self.state.comment_editor_text.push('\n');
            }
            // Annotations
            Action::DeleteAnnotation => {
                if let Some(anchor) = self.selection_to_anchor() {
                    self.state.annotations.delete_at(
                        &anchor.file_path,
                        anchor.line_start,
                        anchor.line_end,
                    );
                    self.set_status("Annotation deleted".to_string(), false);
                }
            }
            Action::NextAnnotation => {
                let file_path = self
                    .state
                    .diff
                    .selected_delta()
                    .map(|d| d.path.to_string_lossy().to_string())
                    .unwrap_or_default();
                // Use current scroll position to approximate current line
                let display_map = self.current_display_map();
                let current_lineno = display_map
                    .get(self.state.diff.scroll_offset)
                    .and_then(|info| info.new_lineno.or(info.old_lineno))
                    .unwrap_or(0);

                if let Some((_next_file, next_line)) = self
                    .state
                    .annotations
                    .next_after(&file_path, current_lineno)
                {
                    // Scroll to the annotation line
                    self.scroll_to_line(next_line);
                }
            }
            Action::PrevAnnotation => {
                let file_path = self
                    .state
                    .diff
                    .selected_delta()
                    .map(|d| d.path.to_string_lossy().to_string())
                    .unwrap_or_default();
                let display_map = self.current_display_map();
                let current_lineno = display_map
                    .get(self.state.diff.scroll_offset)
                    .and_then(|info| info.new_lineno.or(info.old_lineno))
                    .unwrap_or(0);

                if let Some((_prev_file, prev_line)) = self
                    .state
                    .annotations
                    .prev_before(&file_path, current_lineno)
                {
                    self.scroll_to_line(prev_line);
                }
            }

            // Annotation menu
            Action::OpenAnnotationMenu => {
                if let Some(anchor) = self.cursor_to_anchor() {
                    let overlapping = self
                        .state
                        .annotations
                        .annotations_overlapping(&anchor.file_path, anchor.line_start);
                    if overlapping.is_empty() {
                        self.set_status("No annotations on this line".to_string(), false);
                    } else {
                        self.state.annotation_menu_items = overlapping
                            .iter()
                            .map(|a| crate::state::app_state::AnnotationMenuItem {
                                file_path: a.anchor.file_path.clone(),
                                line_start: a.anchor.line_start,
                                line_end: a.anchor.line_end,
                                comment: a.comment.clone(),
                            })
                            .collect();
                        self.state.annotation_menu_selected = 0;
                        self.state.annotation_menu_open = true;
                    }
                }
            }
            Action::AnnotationMenuUp => {
                if !self.state.annotation_menu_items.is_empty() {
                    if self.state.annotation_menu_selected == 0 {
                        self.state.annotation_menu_selected =
                            self.state.annotation_menu_items.len() - 1;
                    } else {
                        self.state.annotation_menu_selected -= 1;
                    }
                }
            }
            Action::AnnotationMenuDown => {
                if !self.state.annotation_menu_items.is_empty() {
                    self.state.annotation_menu_selected = (self.state.annotation_menu_selected + 1)
                        % self.state.annotation_menu_items.len();
                }
            }
            Action::AnnotationMenuDelete => {
                if let Some(item) = self
                    .state
                    .annotation_menu_items
                    .get(self.state.annotation_menu_selected)
                    .cloned()
                {
                    self.state.annotations.delete_annotation(
                        &item.file_path,
                        item.line_start,
                        item.line_end,
                        &item.comment,
                    );
                    self.state
                        .annotation_menu_items
                        .remove(self.state.annotation_menu_selected);
                    if self.state.annotation_menu_items.is_empty() {
                        self.state.annotation_menu_open = false;
                        self.set_status("Annotation deleted".to_string(), false);
                    } else {
                        if self.state.annotation_menu_selected
                            >= self.state.annotation_menu_items.len()
                        {
                            self.state.annotation_menu_selected =
                                self.state.annotation_menu_items.len() - 1;
                        }
                        self.set_status("Annotation deleted".to_string(), false);
                    }
                }
            }
            Action::AnnotationMenuEdit => {
                if let Some(item) = self
                    .state
                    .annotation_menu_items
                    .get(self.state.annotation_menu_selected)
                    .cloned()
                {
                    self.state.editing_annotation =
                        Some(crate::state::app_state::EditingAnnotation {
                            file_path: item.file_path.clone(),
                            line_start: item.line_start,
                            line_end: item.line_end,
                            old_comment: item.comment.clone(),
                        });
                    self.state.annotation_menu_open = false;
                    self.state.comment_editor_open = true;
                    self.state.comment_editor_text = item.comment;
                }
            }
            Action::CancelAnnotationMenu => {
                self.state.annotation_menu_open = false;
                self.state.annotation_menu_items.clear();
            }

            // Prompt / clipboard
            Action::CopyPromptToClipboard => {
                let comment = self.comments_for_file();
                if let Some(rendered) = self.render_prompt_for_selection(&comment) {
                    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&rendered)) {
                        Ok(()) => self.set_status("Prompt copied to clipboard".to_string(), false),
                        Err(e) => {
                            self.set_status(format!("Clipboard error: {e}"), true);
                        }
                    }
                } else {
                    self.set_status("No lines at cursor".to_string(), true);
                }
            }
            Action::TogglePromptPreview => {
                self.state.prompt_preview_visible = !self.state.prompt_preview_visible;
                if self.state.prompt_preview_visible {
                    self.update_prompt_preview();
                }
            }

            // Agent selector
            Action::OpenAgentSelector => {
                if self.config.agents.is_empty() {
                    self.set_status("No agents configured".to_string(), true);
                } else {
                    self.state
                        .agent_selector
                        .last_models
                        .clone_from(&self.config.agent_models);
                    self.state.agent_selector.populate(&self.config.agents);
                    self.state.agent_selector.rerun_prompt = None;
                    self.state.agent_selector.open = true;
                }
            }
            Action::CancelAgentSelector => {
                self.state.agent_selector.open = false;
                self.state.agent_selector.rerun_prompt = None;
            }
            Action::AgentSelectorUp => {
                self.state.agent_selector.select_up();
            }
            Action::AgentSelectorDown => {
                self.state.agent_selector.select_down();
            }
            Action::AgentSelectorFilter(c) => {
                self.state.agent_selector.filter.push(c);
                self.state.agent_selector.refilter();
            }
            Action::AgentSelectorBackspace => {
                self.state.agent_selector.filter.pop();
                self.state.agent_selector.refilter();
            }
            Action::AgentSelectorCycleModel => {
                self.state.agent_selector.cycle_model();
            }
            Action::SelectAgent => {
                let agent = self.state.agent_selector.selected_agent_config().cloned();
                let model = self.state.agent_selector.selected_model_name();
                let rerun_prompt = self.state.agent_selector.rerun_prompt.clone();

                if let (Some(agent), Some(model)) = (agent, model) {
                    // Always use all files + all annotations for the prompt
                    let rendered_prompt =
                        rerun_prompt.or_else(|| self.render_prompt_for_all_files());

                    if let Some(prompt) = rendered_prompt {
                        let command = build_agent_command(&agent.command, &model, &prompt);
                        let run_id = self.state.agent_outputs.next_id;

                        // Use terminal size for PTY, with reasonable defaults
                        let (term_cols, term_rows) =
                            crossterm::terminal::size().unwrap_or((120, 40));
                        // Use ~70% of width for the detail pane
                        let pty_cols = (term_cols * 70 / 100).max(40);
                        let pty_rows = term_rows.saturating_sub(4).max(10);

                        let run = AgentRun {
                            id: run_id,
                            agent_name: agent.name.clone(),
                            model: model.clone(),
                            command: command.clone(),
                            rendered_prompt: prompt,
                            terminal: vt100::Parser::new(pty_rows, pty_cols, 10000),
                            status: AgentRunStatus::Running,
                            started_at: chrono::Utc::now().format("%H:%M").to_string(),
                        };

                        self.state.agent_outputs.add_run(run);
                        self.pty_runner =
                            Some(PtyRunner::spawn(run_id, &command, pty_rows, pty_cols));
                        self.state.agent_selector.open = false;
                        self.state.active_view = ActiveView::AgentOutputs;

                        // Clear annotations — they've been captured in the prompt
                        self.state.annotations = Default::default();
                        session::save_session(
                            &self.repo_path,
                            &self.state.target_label,
                            &self.state.annotations,
                        );

                        // Persist last-used model for this agent
                        self.config
                            .agent_models
                            .insert(agent.name.clone(), model.clone());
                        config::save_agent_model(&agent.name, &model);

                        self.set_status(format!("Running {}/{}", agent.name, model), false);
                    } else {
                        self.set_status("No diff to review".to_string(), true);
                    }
                }
            }

            // Agent outputs tab
            Action::SwitchToAgentOutputs => {
                if self.state.active_view == ActiveView::AgentOutputs {
                    self.state.active_view = ActiveView::DiffExplorer;
                } else {
                    self.state.active_view = ActiveView::AgentOutputs;
                }
            }
            Action::AgentOutputsUp => {
                self.state.agent_outputs.select_up();
            }
            Action::AgentOutputsDown => {
                self.state.agent_outputs.select_down();
            }
            Action::AgentOutputsScrollUp => {
                // detail_scroll is lines from bottom; increase to scroll up
                if let Some(run) = self.state.agent_outputs.selected() {
                    let max_scroll = run.terminal.screen().size().0 as usize;
                    if self.state.agent_outputs.detail_scroll < max_scroll {
                        self.state.agent_outputs.detail_scroll += 1;
                    }
                }
            }
            Action::AgentOutputsScrollDown => {
                // Decrease scroll offset (toward bottom/live output)
                self.state.agent_outputs.detail_scroll =
                    self.state.agent_outputs.detail_scroll.saturating_sub(1);
            }
            Action::AgentOutputsCopyPrompt => {
                if let Some(run) = self.state.agent_outputs.selected() {
                    let prompt = run.rendered_prompt.clone();
                    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&prompt)) {
                        Ok(()) => self.set_status("Prompt copied to clipboard".to_string(), false),
                        Err(e) => {
                            self.set_status(format!("Clipboard error: {e}"), true);
                        }
                    }
                }
            }
            Action::KillAgentProcess => {
                if let Some(runner) = self.pty_runner.as_mut() {
                    runner.kill();
                    self.state.pty_focus = false;
                    self.set_status("Agent process killed".to_string(), false);
                }
            }

            // Review state
            Action::ToggleFileReviewed => {
                if let Some(delta_idx) = self.state.navigator.selected_delta_index() {
                    if let Some(delta) = self.state.diff.deltas.get(delta_idx) {
                        let path = delta.path.to_string_lossy().to_string();
                        self.state.review.toggle_reviewed(&path);
                    }
                }
            }
            Action::NextUnreviewed => {
                use crate::state::review_state::FileReviewStatus;
                let visible = self.state.navigator.visible_entries();
                if visible.is_empty() {
                    return;
                }
                let current = self.state.navigator.selected;
                let len = visible.len();
                // Search from current+1, wrapping around
                for offset in 1..=len {
                    let idx = (current + offset) % len;
                    let path = &visible[idx].1.path;
                    let status = self.state.review.status(path);
                    if matches!(
                        status,
                        FileReviewStatus::Unreviewed
                            | FileReviewStatus::ChangedSinceReview
                            | FileReviewStatus::New
                    ) {
                        self.state.navigator.selected = idx;
                        self.sync_selection();
                        return;
                    }
                }
                self.set_status("All files reviewed".to_string(), false);
            }

            // PTY focus mode
            Action::EnterPtyFocus => {
                // Only enter focus if there's a running agent
                if self.pty_runner.is_some() {
                    if let Some(run) = self.state.agent_outputs.selected() {
                        if matches!(run.status, AgentRunStatus::Running) {
                            self.state.pty_focus = true;
                        }
                    }
                }
            }
            Action::ExitPtyFocus => {
                self.state.pty_focus = false;
            }
            Action::PtyInput(key) => {
                if let Some(runner) = self.pty_runner.as_mut() {
                    let bytes = key_event_to_bytes(&key);
                    if !bytes.is_empty() {
                        runner.write_input(&bytes);
                    }
                }
            }

            Action::RefreshDiff => {
                self.request_diff();
                self.set_status("Refreshed".to_string(), false);
            }

            Action::ToggleHud => {
                self.state.hud_expanded = !self.state.hud_expanded;
                // 10 seconds at 50ms tick rate
                self.hud_collapse_countdown = if self.state.hud_expanded { 200 } else { 0 };
            }

            Action::Tick => {
                if self.status_clear_countdown > 0 {
                    self.status_clear_countdown -= 1;
                    if self.status_clear_countdown == 0 {
                        self.state.status_message = None;
                    }
                }
                if self.hud_collapse_countdown > 0 {
                    self.hud_collapse_countdown -= 1;
                    if self.hud_collapse_countdown == 0 {
                        self.state.hud_expanded = false;
                    }
                }
            }
            Action::ExpandContext => {
                let display_map = self.current_display_map();
                if let Some(info) = display_map.get(self.state.diff.cursor_row) {
                    if info.is_collapsed_indicator {
                        if let Some(gap_id) = info.gap_id {
                            let current = self
                                .state
                                .diff
                                .gap_expansions
                                .get(&gap_id)
                                .copied()
                                .unwrap_or(0);
                            self.state.diff.gap_expansions.insert(gap_id, current + 20);
                        }
                    }
                }
            }
            // Settings modal
            Action::OpenSettings => {
                self.state.settings.open = true;
                self.state.settings.selected_row = 0;
            }
            Action::CloseSettings => {
                self.state.settings.open = false;
                // Persist all settings to config.toml
                config::save_settings(&PersistentSettings {
                    theme: self.state.theme.name.clone(),
                    unified: self.state.diff.options.view_mode == DiffViewMode::Unified,
                    ignore_whitespace: self.state.diff.options.ignore_whitespace,
                    context_lines: self.state.diff.display_context,
                });
            }
            Action::SettingsUp => {
                if self.state.settings.selected_row > 0 {
                    self.state.settings.selected_row -= 1;
                }
            }
            Action::SettingsDown => {
                if self.state.settings.selected_row < SETTINGS_ROW_COUNT - 1 {
                    self.state.settings.selected_row += 1;
                }
            }
            Action::SettingsLeft => {
                match self.state.settings.selected_row {
                    0 => {
                        // Prev theme
                        let new_name = prev_theme(&self.state.theme.name);
                        self.state.theme = Theme::from_name(new_name);
                        self.update_highlights();
                    }
                    1 => {
                        // Toggle view mode
                        self.state.diff.options.view_mode = match self.state.diff.options.view_mode
                        {
                            DiffViewMode::Split => DiffViewMode::Unified,
                            DiffViewMode::Unified => DiffViewMode::Split,
                        };
                        self.state.diff.cursor_row = 0;
                        self.state.diff.scroll_offset = 0;
                        self.state.selection.active = false;
                    }
                    2 => {
                        // Toggle whitespace
                        self.state.diff.options.ignore_whitespace =
                            !self.state.diff.options.ignore_whitespace;
                        self.request_diff();
                    }
                    3 => {
                        // Decrease context lines (min 1)
                        if self.state.diff.display_context > 1 {
                            self.state.diff.display_context -= 1;
                        }
                    }
                    _ => {}
                }
            }
            Action::SettingsRight => {
                match self.state.settings.selected_row {
                    0 => {
                        // Next theme
                        let new_name = next_theme(&self.state.theme.name);
                        self.state.theme = Theme::from_name(new_name);
                        self.update_highlights();
                    }
                    1 => {
                        // Toggle view mode
                        self.state.diff.options.view_mode = match self.state.diff.options.view_mode
                        {
                            DiffViewMode::Split => DiffViewMode::Unified,
                            DiffViewMode::Unified => DiffViewMode::Split,
                        };
                        self.state.diff.cursor_row = 0;
                        self.state.diff.scroll_offset = 0;
                        self.state.selection.active = false;
                    }
                    2 => {
                        // Toggle whitespace
                        self.state.diff.options.ignore_whitespace =
                            !self.state.diff.options.ignore_whitespace;
                        self.request_diff();
                    }
                    3 => {
                        // Increase context lines (max 20)
                        if self.state.diff.display_context < 20 {
                            self.state.diff.display_context += 1;
                        }
                    }
                    _ => {}
                }
            }

            Action::Resize => {
                // Resize PTY and active terminal parser to match new terminal size
                if let Some(runner) = self.pty_runner.as_ref() {
                    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((120, 40));
                    let pty_cols = (term_cols * 70 / 100).max(40);
                    let pty_rows = term_rows.saturating_sub(4).max(10);
                    runner.resize(pty_rows, pty_cols);
                    // Resize the terminal parser for the running agent
                    if let Some(run) = self
                        .state
                        .agent_outputs
                        .runs
                        .iter_mut()
                        .find(|r| matches!(r.status, AgentRunStatus::Running))
                    {
                        run.terminal.set_size(pty_rows, pty_cols);
                    }
                }
            }
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

    /// Validate a ref string against the repo. Returns the ComparisonTarget and a display label.
    fn validate_ref(&self, input: &str) -> Result<(ComparisonTarget, String), String> {
        let repo =
            git2::Repository::open(&self.repo_path).map_err(|e| format!("open repo: {e}"))?;
        repo.revparse_single(input).map_err(|e| format!("{e}"))?;
        // Use parse_target for consistent ComparisonTarget construction
        let target = parse_target(Some(input));
        let label = match &target {
            ComparisonTarget::HeadVsWorkdir => "HEAD".to_string(),
            ComparisonTarget::Branch(name) => name.clone(),
            ComparisonTarget::Commit(oid) => format!("{:.7}", oid),
        };
        Ok((target, label))
    }

    /// Switch to a new comparison target, preserving annotations per-target.
    fn apply_new_target(&mut self, target: ComparisonTarget, label: String) {
        // Save current session
        session::save_session(
            &self.repo_path,
            &self.state.target_label,
            &self.state.annotations,
        );

        // Update target
        self.target = target;
        self.state.target_label = label.clone();

        // Load annotations for the new target
        self.state.annotations = session::load_session(&self.repo_path, &label);

        // Reset diff/navigator/review state
        self.state.diff.deltas.clear();
        self.state.diff.selected_file = None;
        self.state.diff.scroll_offset = 0;
        self.state.diff.cursor_row = 0;
        self.state.navigator.entries.clear();
        self.state.navigator.filtered_indices.clear();
        self.state.selection.active = false;
        self.state.review.reset();

        self.request_diff();
        self.set_status(format!("Target: {label}"), false);
    }

    /// Mark the current file as reviewed if the cursor has reached the last row
    /// and the diff view is focused.
    fn check_auto_review(&mut self) {
        if self.state.focus != FocusPanel::DiffView {
            return;
        }
        let max = self.current_display_map().len().saturating_sub(1);
        if self.state.diff.cursor_row >= max {
            if let Some(delta) = self.state.diff.selected_delta() {
                let path = delta.path.to_string_lossy().to_string();
                self.state.review.mark_reviewed(&path);
            }
        }
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
                self.state.diff.cursor_row = 0;
                self.update_highlights();
                // Exit visual mode when switching files
                self.state.selection.active = false;
                // Reset context expansions for the new file
                self.state.diff.gap_expansions.clear();
            }
        }
    }

    /// Scroll to the display row containing the given line number.
    fn scroll_to_line(&mut self, target_lineno: u32) {
        let display_map = self.current_display_map();
        for (row_idx, info) in display_map.iter().enumerate() {
            let matches =
                info.new_lineno == Some(target_lineno) || info.old_lineno == Some(target_lineno);
            if matches {
                self.state.diff.scroll_offset = row_idx;
                return;
            }
        }
    }

    /// Collect all annotations for the current file, formatted with line ranges.
    fn comments_for_file(&self) -> String {
        let file_path = self
            .state
            .diff
            .selected_delta()
            .map(|d| d.path.to_string_lossy().to_string());
        let Some(file_path) = file_path else {
            return String::new();
        };
        let Some(anns) = self.state.annotations.annotations.get(&file_path) else {
            return String::new();
        };
        if anns.is_empty() {
            return String::new();
        }

        anns.iter()
            .map(|ann| {
                if ann.anchor.line_start == ann.anchor.line_end {
                    format!("- Line {}: {}", ann.anchor.line_start, ann.comment)
                } else {
                    format!(
                        "- Lines {}-{}: {}",
                        ann.anchor.line_start, ann.anchor.line_end, ann.comment
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Collect all annotations across all files, formatted with file paths and line ranges.
    fn comments_for_all_files(&self) -> String {
        let mut parts = Vec::new();
        for (file_path, anns) in &self.state.annotations.annotations {
            for ann in anns {
                if ann.anchor.line_start == ann.anchor.line_end {
                    parts.push(format!(
                        "- {} Line {}: {}",
                        file_path, ann.anchor.line_start, ann.comment
                    ));
                } else {
                    parts.push(format!(
                        "- {} Lines {}-{}: {}",
                        file_path, ann.anchor.line_start, ann.anchor.line_end, ann.comment
                    ));
                }
            }
        }
        parts.join("\n")
    }

    /// Render a prompt covering all files in the diff with all annotations.
    fn render_prompt_for_all_files(&self) -> Option<String> {
        if self.state.diff.deltas.is_empty() {
            return None;
        }

        let comments = self.comments_for_all_files();

        let mut diff_sections = Vec::new();
        for delta in &self.state.diff.deltas {
            let filename = delta.path.to_string_lossy();
            let mut lines = Vec::new();
            for hunk in &delta.hunks {
                if !hunk.header.is_empty() {
                    lines.push(hunk.header.trim_end().to_string());
                }
                for line in &hunk.lines {
                    let prefix = match line.origin {
                        DiffLineOrigin::Addition => "+",
                        DiffLineOrigin::Deletion => "-",
                        DiffLineOrigin::Context => " ",
                    };
                    lines.push(format!("{}{}", prefix, line.content.trim_end()));
                }
            }
            diff_sections.push(format!(
                "### {}\n```diff\n{}\n```",
                filename,
                lines.join("\n")
            ));
        }

        let prompt = format!(
            "You are reviewing a code change. A reviewer has left comments on the diff below. \
             Address each review comment by making the necessary code changes. If a comment asks \
             a question, answer it and make any implied fixes. Keep changes minimal and focused \
             on what the reviewer asked for.\n\n\
             ## Review Comments\n\n{}\n\n\
             ## Changes\n\n{}",
            comments,
            diff_sections.join("\n\n")
        );
        Some(prompt)
    }

    /// Update the prompt preview text from the current selection state.
    fn update_prompt_preview(&mut self) {
        let comment = self.comments_for_file();
        self.state.prompt_preview_text = self
            .render_prompt_for_selection(&comment)
            .unwrap_or_default();
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

/// Build the shell command for an agent by substituting `{model}` and `{rendered_prompt}`.
fn build_agent_command(command_template: &str, model: &str, prompt: &str) -> String {
    // Escape single quotes in the prompt for safe shell embedding
    let escaped_prompt = prompt.replace('\'', "'\\''");
    command_template
        .replace("{model}", model)
        .replace("{rendered_prompt}", &escaped_prompt)
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
