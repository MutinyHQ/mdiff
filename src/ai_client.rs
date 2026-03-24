use anyhow::{anyhow, Result};

use crate::config::{AgenticReviewConfig, ApiKeysConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Moonshot,
}

impl AiProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            "moonshot" => Ok(Self::Moonshot),
            _ => Err(anyhow!("Unknown provider: {s}")),
        }
    }

    pub fn env_var_name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Moonshot => "MOONSHOT_API_KEY",
        }
    }
}

/// Resolve API key: env var first, then config.toml [api_keys].
pub fn resolve_api_key(provider: &AiProvider, api_keys: &Option<ApiKeysConfig>) -> Option<String> {
    if let Ok(key) = std::env::var(provider.env_var_name()) {
        if !key.is_empty() {
            return Some(key);
        }
    }

    if let Some(keys) = api_keys {
        let val = match provider {
            AiProvider::OpenAI => &keys.openai,
            AiProvider::Anthropic => &keys.anthropic,
            AiProvider::Moonshot => &keys.moonshot,
        };
        if let Some(key) = val {
            return Some(key.clone());
        }
    }

    None
}

/// Resolve base URL override from config. Returns None if default should be used.
/// Only returns a value when the user has explicitly set a custom base_url.
pub fn resolve_base_url_override(config: &AgenticReviewConfig) -> Option<String> {
    config.base_url.clone()
}
