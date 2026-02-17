use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub prompt_template: String,
    pub context_padding: usize,
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
            prompt_template: DEFAULT_TEMPLATE.to_string(),
            context_padding: 5,
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
            command: "claude -p --model {model} '{rendered_prompt}'".to_string(),
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
            command: "codex --message '{rendered_prompt}'".to_string(),
            models: vec![],
            default_model: String::new(),
            description: "OpenAI Codex CLI".to_string(),
        },
        AgentProviderConfig {
            name: "opencode".to_string(),
            command: "opencode run --model {model} --prompt '{rendered_prompt}'".to_string(),
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
            command: "gemini -p '{rendered_prompt}'".to_string(),
            models: vec!["gemini-2.5-pro".to_string(), "gemini-2.5-flash".to_string()],
            default_model: "gemini-2.5-pro".to_string(),
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

const DEFAULT_TEMPLATE: &str = r#"You are reviewing a code change. A reviewer has left comments on the diff below. Address each review comment by making the necessary code changes. If a comment asks a question, answer it and make any implied fixes. Keep changes minimal and focused on what the reviewer asked for.

## Review Comments

{comments}

## File

{filename} (Lines {line_start}-{line_end})

{hunk_header}

```diff
{diff_content}
```

## Surrounding Context

{context}"#;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default = "default_context_padding")]
    context_padding: usize,
    #[serde(default)]
    prompt_template: Option<String>,
    #[serde(default)]
    agents: Vec<AgentProviderConfig>,
}

fn default_context_padding() -> usize {
    5
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

    MdiffConfig {
        agents,
        agents_by_name,
        prompt_template: file
            .prompt_template
            .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string()),
        context_padding: file.context_padding,
    }
}
