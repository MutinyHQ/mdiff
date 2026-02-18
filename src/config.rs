use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::theme::{apply_overrides, Theme, ThemeOverrides};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AgentProviderConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub default_model: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MdiffConfig {
    pub agents: Vec<AgentProviderConfig>,
    pub agents_by_name: HashMap<String, usize>,
    pub theme: Theme,
    pub unified: Option<bool>,
    pub ignore_whitespace: Option<bool>,
    pub context_lines: Option<usize>,
    /// Last-used model per agent name (e.g. "claude" -> "claude-opus-4-6").
    pub agent_models: HashMap<String, String>,
}

impl Default for MdiffConfig {
    fn default() -> Self {
        let agents = detect_agents();
        let agents_by_name = agents
            .iter()
            .enumerate()
            .map(|(i, a)| (a.name.clone(), i))
            .collect();
        Self {
            agents,
            agents_by_name,
            theme: Theme::from_name("one-dark"),
            unified: None,
            ignore_whitespace: None,
            context_lines: None,
            agent_models: HashMap::new(),
        }
    }
}

/// Check if an executable exists on PATH.
fn has_command(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Known CLI agents with their default configurations.
fn known_agents() -> Vec<AgentProviderConfig> {
    vec![
        AgentProviderConfig {
            name: "claude".to_string(),
            command: "claude --permission-mode acceptEdits --model {model} '{rendered_prompt}'"
                .to_string(),
            models: vec![
                "claude-sonnet-4-6".to_string(),
                "claude-opus-4-6".to_string(),
                "claude-haiku-4-5".to_string(),
            ],
            default_model: "claude-sonnet-4-6".to_string(),
            description: "Anthropic Claude Code".to_string(),
        },
        AgentProviderConfig {
            name: "codex".to_string(),
            command:
                "codex --sandbox workspace-write --ask-for-approval untrusted '{rendered_prompt}'"
                    .to_string(),
            models: vec![],
            default_model: String::new(),
            description: "OpenAI Codex CLI".to_string(),
        },
        AgentProviderConfig {
            name: "opencode".to_string(),
            command: "opencode -m {model} '{rendered_prompt}'".to_string(),
            models: vec![
                "anthropic/claude-sonnet-4-6".to_string(),
                "openai/gpt-5.2-codex".to_string(),
                "openai/o3".to_string(),
            ],
            default_model: "anthropic/claude-sonnet-4-6".to_string(),
            description: "OpenCode CLI".to_string(),
        },
        AgentProviderConfig {
            name: "gemini".to_string(),
            command: "gemini --approval-mode auto_edit '{rendered_prompt}'".to_string(),
            models: vec![
                "gemini-3-flash-preview".to_string(),
                "gemini-3-pro-preview".to_string(),
                "gemini-2.5-pro".to_string(),
                "gemini-2.5-flash".to_string(),
            ],
            default_model: "gemini-3-flash-preview".to_string(),
            description: "Google Gemini CLI".to_string(),
        },
    ]
}

/// Auto-detect which known agent CLIs are available on PATH.
fn detect_agents() -> Vec<AgentProviderConfig> {
    known_agents()
        .into_iter()
        .filter(|a| has_command(&a.name))
        .collect()
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    agents: Vec<AgentProviderConfig>,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    colors: Option<ThemeOverrides>,
    #[serde(default)]
    unified: Option<bool>,
    #[serde(default)]
    ignore_whitespace: Option<bool>,
    #[serde(default)]
    context_lines: Option<usize>,
    #[serde(default)]
    agent_models: HashMap<String, String>,
}

fn config_path() -> PathBuf {
    let mut path = dirs_home().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("mdiff");
    path.push("config.toml");
    path
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Build the agents_by_name index from an agents list.
fn build_agents_index(agents: &[AgentProviderConfig]) -> HashMap<String, usize> {
    agents
        .iter()
        .enumerate()
        .map(|(i, a)| (a.name.clone(), i))
        .collect()
}

/// Load config from `~/.config/mdiff/config.toml`, falling back to defaults.
/// If no agents are configured, auto-detects known CLIs on PATH.
pub fn load_config() -> MdiffConfig {
    let path = config_path();

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return MdiffConfig::default(),
    };

    let file: ConfigFile = match toml::from_str(&contents) {
        Ok(f) => f,
        Err(_) => return MdiffConfig::default(),
    };

    // Use configured agents, or fall back to auto-detection
    let agents = if file.agents.is_empty() {
        detect_agents()
    } else {
        file.agents
    };

    let agents_by_name = build_agents_index(&agents);

    // Load theme by name, apply color overrides
    let theme_name = file.theme.as_deref().unwrap_or("one-dark");
    let mut theme = Theme::from_name(theme_name);
    if let Some(ref overrides) = file.colors {
        apply_overrides(&mut theme, overrides);
    }

    MdiffConfig {
        agents,
        agents_by_name,
        theme,
        unified: file.unified,
        ignore_whitespace: file.ignore_whitespace,
        context_lines: file.context_lines,
        agent_models: file.agent_models,
    }
}

/// Settings that get persisted to config.toml when the settings modal closes.
pub struct PersistentSettings {
    pub theme: String,
    pub unified: bool,
    pub ignore_whitespace: bool,
    pub context_lines: usize,
}

/// Save persistent settings to `~/.config/mdiff/config.toml`.
/// Reads the existing file (if any), updates only the settings fields, and writes back.
/// Preserves other config values (agents, prompt_template, color overrides).
pub fn save_settings(settings: &PersistentSettings) {
    let path = config_path();

    // Read existing config as a TOML table to preserve unknown fields
    let mut table = if let Ok(contents) = std::fs::read_to_string(&path) {
        contents
            .parse::<toml::Table>()
            .unwrap_or_else(|_| toml::Table::new())
    } else {
        toml::Table::new()
    };

    table.insert(
        "theme".to_string(),
        toml::Value::String(settings.theme.clone()),
    );
    table.insert(
        "unified".to_string(),
        toml::Value::Boolean(settings.unified),
    );
    table.insert(
        "ignore_whitespace".to_string(),
        toml::Value::Boolean(settings.ignore_whitespace),
    );
    table.insert(
        "context_lines".to_string(),
        toml::Value::Integer(settings.context_lines as i64),
    );

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let toml_string = toml::to_string_pretty(&table).unwrap_or_default();
    let _ = std::fs::write(&path, toml_string);
}

/// Save the last-used model for a specific agent to config.toml.
pub fn save_agent_model(agent_name: &str, model: &str) {
    let path = config_path();

    let mut table = if let Ok(contents) = std::fs::read_to_string(&path) {
        contents
            .parse::<toml::Table>()
            .unwrap_or_else(|_| toml::Table::new())
    } else {
        toml::Table::new()
    };

    let agent_models = table
        .entry("agent_models")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    if let toml::Value::Table(ref mut t) = agent_models {
        t.insert(
            agent_name.to_string(),
            toml::Value::String(model.to_string()),
        );
    }

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let toml_string = toml::to_string_pretty(&table).unwrap_or_default();
    let _ = std::fs::write(&path, toml_string);
}
