use crate::config::AgentProviderConfig;

/// Status of an agent process run.
#[derive(Debug, Clone)]
pub enum AgentRunStatus {
    Running,
    Success { exit_code: i32 },
    Failed { exit_code: i32 },
}

/// A single agent execution run.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AgentRun {
    pub id: usize,
    pub agent_name: String,
    pub model: String,
    pub command: String,
    pub rendered_prompt: String,
    pub output_lines: Vec<String>,
    pub status: AgentRunStatus,
    pub started_at: String,
    pub source_file: String,
    pub source_lines: (u32, u32),
}

/// State for the agent outputs tab.
#[derive(Debug, Default)]
pub struct AgentOutputsState {
    pub runs: Vec<AgentRun>,
    pub selected_run: usize,
    pub detail_scroll: usize,
    pub next_id: usize,
}

impl AgentOutputsState {
    pub fn add_run(&mut self, run: AgentRun) {
        self.runs.insert(0, run);
        self.selected_run = 0;
        self.detail_scroll = 0;
        self.next_id += 1;
    }

    pub fn selected(&self) -> Option<&AgentRun> {
        self.runs.get(self.selected_run)
    }

    pub fn select_up(&mut self) {
        self.selected_run = self.selected_run.saturating_sub(1);
        self.detail_scroll = 0;
    }

    pub fn select_down(&mut self) {
        if !self.runs.is_empty() {
            self.selected_run = (self.selected_run + 1).min(self.runs.len() - 1);
            self.detail_scroll = 0;
        }
    }
}

/// State for the agent selector modal.
#[derive(Debug, Default)]
pub struct AgentSelectorState {
    pub open: bool,
    pub filter: String,
    pub selected_agent: usize,
    pub selected_model: usize,
    pub agents: Vec<AgentProviderConfig>,
    pub filtered_indices: Vec<usize>,
    pub rerun_prompt: Option<String>,
}

impl AgentSelectorState {
    /// Populate agents from config and reset filter.
    pub fn populate(&mut self, agents: &[AgentProviderConfig]) {
        self.agents = agents.to_vec();
        self.filter.clear();
        self.selected_agent = 0;
        self.selected_model = 0;
        self.refilter();
    }

    pub fn refilter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.agents.len()).collect();
        } else {
            let query = self.filter.to_lowercase();
            self.filtered_indices = self
                .agents
                .iter()
                .enumerate()
                .filter(|(_, a)| a.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect();
        }
        if !self.filtered_indices.is_empty() {
            self.selected_agent = self.selected_agent.min(self.filtered_indices.len() - 1);
        } else {
            self.selected_agent = 0;
        }
    }

    /// Get the currently selected agent config, if any.
    pub fn selected_agent_config(&self) -> Option<&AgentProviderConfig> {
        self.filtered_indices
            .get(self.selected_agent)
            .and_then(|&i| self.agents.get(i))
    }

    /// Get the currently selected model name for the selected agent.
    pub fn selected_model_name(&self) -> Option<String> {
        let agent = self.selected_agent_config()?;
        if agent.models.is_empty() {
            Some(agent.default_model.clone())
        } else {
            Some(
                agent
                    .models
                    .get(self.selected_model)
                    .cloned()
                    .unwrap_or_else(|| agent.default_model.clone()),
            )
        }
    }

    /// Cycle to the next model for the currently selected agent.
    pub fn cycle_model(&mut self) {
        if let Some(agent) = self.selected_agent_config() {
            if !agent.models.is_empty() {
                // Clone the len to avoid borrow conflict
                let len = agent.models.len();
                self.selected_model = (self.selected_model + 1) % len;
            }
        }
    }

    pub fn select_up(&mut self) {
        self.selected_agent = self.selected_agent.saturating_sub(1);
        self.selected_model = 0;
    }

    pub fn select_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_agent = (self.selected_agent + 1).min(self.filtered_indices.len() - 1);
            self.selected_model = 0;
        }
    }
}
