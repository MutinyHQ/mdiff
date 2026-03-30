use anyhow::Result;
use futures::StreamExt;
use rig::client::CompletionClient;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use tokio::sync::{mpsc, oneshot};

use crate::ai_client::{self, AiProvider};
use crate::config::{AgenticReviewConfig, ApiKeysConfig};
use crate::state::qa_state::QuestionContext;

const QA_SYSTEM_PROMPT: &str = "\
You are a code review assistant. The user is reviewing a git diff and has a question about the changes.

Provide a concise, helpful answer. Focus on explaining the code changes and their intent. \
Keep the response under 500 words unless the question requires more detail.";

#[derive(Debug)]
pub enum QAEvent {
    Token(String),
    Complete,
    Error(String),
}

pub struct QARunner {
    event_rx: mpsc::UnboundedReceiver<QAEvent>,
    _kill_tx: oneshot::Sender<()>,
}

impl QARunner {
    pub fn try_recv(&mut self) -> Option<QAEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn spawn(
        context: QuestionContext,
        config: AgenticReviewConfig,
        api_keys: Option<ApiKeysConfig>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (kill_tx, kill_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            let result = run_qa(context, config, api_keys, &event_tx, kill_rx).await;
            match result {
                Ok(()) => {
                    let _ = event_tx.send(QAEvent::Complete);
                }
                Err(e) => {
                    let _ = event_tx.send(QAEvent::Error(e.to_string()));
                }
            }
        });

        Self {
            event_rx,
            _kill_tx: kill_tx,
        }
    }
}

fn build_user_prompt(ctx: &QuestionContext) -> String {
    let mut parts = Vec::new();

    parts.push(format!("File: {}", ctx.file_path));

    if let Some(ref lang) = ctx.file_language {
        parts.push(format!("Language: {lang}"));
    }

    if !ctx.full_diff.is_empty() {
        let diff_preview = if ctx.full_diff.len() > 8000 {
            format!("{}...(truncated)", &ctx.full_diff[..8000])
        } else {
            ctx.full_diff.clone()
        };
        parts.push(format!("Diff:\n```diff\n{diff_preview}\n```"));
    }

    if let Some(ref selected) = ctx.selected_lines {
        parts.push(format!("Selected lines:\n```\n{selected}\n```"));
    }

    if !ctx.visible_hunks.is_empty() {
        let hunks = ctx.visible_hunks.join("\n---\n");
        let hunks_preview = if hunks.len() > 4000 {
            format!("{}...(truncated)", &hunks[..4000])
        } else {
            hunks
        };
        parts.push(format!("Visible hunks:\n```diff\n{hunks_preview}\n```"));
    }

    parts.push(format!("User's question: {}", ctx.question));

    parts.join("\n\n")
}

macro_rules! maybe_base_url {
    ($builder:expr, $base_url:expr) => {
        if let Some(ref url) = $base_url {
            $builder.base_url(url)
        } else {
            $builder
        }
    };
}

/// Process a single stream item, sending token events. Returns false on error.
macro_rules! drain_stream {
    ($stream_result:expr, $event_tx:expr) => {{
        let mut stream = std::pin::pin!($stream_result);
        while let Some(item) = stream.next().await {
            match item {
                Ok(rig::agent::MultiTurnStreamItem::StreamAssistantItem(content)) => {
                    if let StreamedAssistantContent::Text(text) = content {
                        let _ = $event_tx.send(QAEvent::Token(text.text));
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = $event_tx.send(QAEvent::Error(format!("Stream error: {e}")));
                    break;
                }
            }
        }
    }};
}

async fn run_qa(
    context: QuestionContext,
    config: AgenticReviewConfig,
    api_keys: Option<ApiKeysConfig>,
    event_tx: &mpsc::UnboundedSender<QAEvent>,
    kill_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let provider = AiProvider::from_str(config.resolved_parent_provider())?;
    let api_key = ai_client::resolve_api_key(&provider, &api_keys).ok_or_else(|| {
        anyhow::anyhow!("Missing API key for {}", config.resolved_parent_provider())
    })?;

    let base_url_override = ai_client::resolve_base_url_override(&config);
    let user_prompt = build_user_prompt(&context);

    match provider {
        AiProvider::Anthropic => {
            let builder = rig::providers::anthropic::Client::builder().api_key(&api_key);
            let client = maybe_base_url!(builder, base_url_override)
                .build()
                .map_err(|e| anyhow::anyhow!("Anthropic client error: {}", e))?;

            let agent = client
                .agent(&config.parent_model)
                .preamble(QA_SYSTEM_PROMPT)
                .build();

            let stream_result = tokio::select! {
                s = agent.stream_prompt(&user_prompt) => s,
                _ = kill_rx => return Ok(()),
            };

            drain_stream!(stream_result, event_tx);
        }
        AiProvider::OpenAI | AiProvider::Moonshot => {
            let builder = rig::providers::openai::Client::builder().api_key(&api_key);
            let client = maybe_base_url!(builder, base_url_override)
                .build()
                .map_err(|e| anyhow::anyhow!("OpenAI client error: {}", e))?;

            let agent = client
                .agent(&config.parent_model)
                .preamble(QA_SYSTEM_PROMPT)
                .build();

            let stream_result = tokio::select! {
                s = agent.stream_prompt(&user_prompt) => s,
                _ = kill_rx => return Ok(()),
            };

            drain_stream!(stream_result, event_tx);
        }
    }

    Ok(())
}
