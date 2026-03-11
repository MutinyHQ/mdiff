# Spec: Add New Opencode Review Models (Issue #37)

**Priority**: P1 (Feature Request)
**Status**: Ready for implementation
**Estimated effort**: Small (1 file changed)
**Addresses**: GitHub Issue #37

## Problem

The `opencode` agent provider in mdiff's `known_agents()` configuration only includes three models: `anthropic/claude-sonnet-4-6`, `openai/gpt-5.2-codex`, and `openai/o3`. The user has requested adding `gpt-5.4` and `gpt-5.3-codex` to both the `opencode` and `codex` agent providers so they can use the latest OpenAI models for AI-powered code review.

## Solution

### File: `src/config.rs`

Update the `known_agents()` function to add the new models:

#### 1. Update the `opencode` provider

Add `openai/gpt-5.4` and `openai/gpt-5.3-codex` to the `opencode` models list. Place them at the top of the list since they are the newest models:

```rust
AgentProviderConfig {
    name: "opencode".to_string(),
    command: "opencode -m {model} --prompt '{rendered_prompt}'".to_string(),
    models: vec![
        "openai/gpt-5.4".to_string(),
        "openai/gpt-5.3-codex".to_string(),
        "anthropic/claude-sonnet-4-6".to_string(),
        "openai/gpt-5.2-codex".to_string(),
        "openai/o3".to_string(),
    ],
    default_model: "anthropic/claude-sonnet-4-6".to_string(),
    description: "OpenCode CLI".to_string(),
},
```

#### 2. Update the `codex` provider

Add models to the `codex` provider, which currently has an empty models list. The issue mentions adding models to "opencode and codex":

```rust
AgentProviderConfig {
    name: "codex".to_string(),
    command: "codex --sandbox workspace-write --ask-for-approval untrusted '{rendered_prompt}'"
        .to_string(),
    models: vec![
        "gpt-5.4".to_string(),
        "gpt-5.3-codex".to_string(),
        "gpt-5.2-codex".to_string(),
    ],
    default_model: "gpt-5.4".to_string(),
    description: "OpenAI Codex CLI".to_string(),
},
```

Note: The `codex` CLI uses model names without the `openai/` prefix (unlike `opencode` which uses provider-prefixed names). The `codex` command template does not currently include a `{model}` placeholder — the command string should be updated to pass the model:

```rust
command: "codex --model {model} --sandbox workspace-write --ask-for-approval untrusted '{rendered_prompt}'"
    .to_string(),
```

### Important: Preserve user config

Users who have configured custom agents in `~/.config/mdiff/config.toml` will not be affected, since `load_config()` only uses `known_agents()` / `detect_agents()` when no agents are configured in the file. This change only affects the auto-detected defaults.

## Files Changed

- `src/config.rs` — Update `known_agents()` to add new models to `opencode` and `codex` providers, update `codex` command to include `{model}` placeholder

## Testing

1. `cargo check` passes
2. Run mdiff, open agent selector — verify `opencode` shows the 5 models in correct order
3. Verify `codex` shows the 3 models
4. Cycle through models with the model cycle key — verify all new models appear
5. Select `gpt-5.4` for opencode, close and reopen — verify last-used model is persisted
