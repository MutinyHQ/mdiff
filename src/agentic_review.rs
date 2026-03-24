use anyhow::Result;
use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

use crate::ai_client::{self, AiProvider};
use crate::config::{AgenticReviewConfig, ApiKeysConfig};
use crate::git::types::{DiffLineOrigin, FileDelta};
use crate::review_tools::{
    AnnotateFileArgs, AnnotateFileTool, CreateAnnotationArgs, CreateAnnotationTool, ListFilesTool,
    ReadFileTool, SearchTool,
};
use crate::state::annotation_state::{
    Annotation, AnnotationCategory, AnnotationSeverity, LineAnchor,
};

/// Events sent from the orchestration runner to the main loop.
#[derive(Debug)]
pub enum AgenticReviewEvent {
    StreamToken(String),
    ChildProgress(usize, usize),
    Complete(Vec<Annotation>),
    Error(String),
}

/// The orchestration runner.
/// Dropping the runner cancels the pipeline via the kill channel.
pub struct AgenticReviewRunner {
    event_rx: mpsc::UnboundedReceiver<AgenticReviewEvent>,
    // Held to keep the channel open — dropping cancels the pipeline via kill_rx
    _kill_tx: oneshot::Sender<()>,
}

impl AgenticReviewRunner {
    pub fn try_recv(&mut self) -> Option<AgenticReviewEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn spawn(
        review_text: String,
        deltas: Vec<FileDelta>,
        config: AgenticReviewConfig,
        api_keys: Option<ApiKeysConfig>,
        repo_path: PathBuf,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (kill_tx, kill_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            let result = run_pipeline(
                review_text,
                deltas,
                config,
                api_keys,
                repo_path,
                &event_tx,
                kill_rx,
            )
            .await;
            if let Err(e) = result {
                let _ = event_tx.send(AgenticReviewEvent::Error(e.to_string()));
            }
        });

        Self {
            event_rx,
            _kill_tx: kill_tx,
        }
    }
}

// ================================================================
// Helpers
// ================================================================

fn render_diff_text(deltas: &[FileDelta]) -> String {
    let mut out = String::new();
    for delta in deltas {
        if delta.binary {
            continue;
        }
        let path = delta.path.to_string_lossy();
        out.push_str(&format!("--- a/{path}\n+++ b/{path}\n"));
        for hunk in &delta.hunks {
            out.push_str(&hunk.header);
            if !hunk.header.ends_with('\n') {
                out.push('\n');
            }
            for line in &hunk.lines {
                let prefix = match line.origin {
                    DiffLineOrigin::Addition => "+",
                    DiffLineOrigin::Deletion => "-",
                    DiffLineOrigin::Context => " ",
                };
                out.push_str(&format!("{prefix}{}\n", line.content.trim_end()));
            }
        }
        out.push('\n');
    }
    out
}

fn render_file_diff(delta: &FileDelta) -> String {
    render_diff_text(std::slice::from_ref(delta))
}

fn parse_category(s: &str) -> AnnotationCategory {
    match s {
        "Bug" => AnnotationCategory::Bug,
        "Style" => AnnotationCategory::Style,
        "Performance" => AnnotationCategory::Performance,
        "Security" => AnnotationCategory::Security,
        "Suggestion" => AnnotationCategory::Suggestion,
        "Question" => AnnotationCategory::Question,
        "Nitpick" => AnnotationCategory::Nitpick,
        _ => AnnotationCategory::Suggestion,
    }
}

fn parse_severity(s: &str) -> AnnotationSeverity {
    match s {
        "Critical" => AnnotationSeverity::Critical,
        "Major" => AnnotationSeverity::Major,
        "Minor" => AnnotationSeverity::Minor,
        "Info" => AnnotationSeverity::Info,
        _ => AnnotationSeverity::Minor,
    }
}

/// Conditionally apply base_url to a rig client builder.
macro_rules! maybe_base_url {
    ($builder:expr, $base_url:expr) => {
        if let Some(ref url) = $base_url {
            $builder.base_url(url)
        } else {
            $builder
        }
    };
}

/// Macro to add shared tools (list_files, read_file, search) to an agent builder.
macro_rules! with_shared_tools {
    ($builder:expr, $repo_path:expr) => {
        $builder
            .tool(ListFilesTool {
                repo_path: $repo_path.clone(),
            })
            .tool(ReadFileTool {
                repo_path: $repo_path.clone(),
            })
            .tool(SearchTool {
                repo_path: $repo_path.clone(),
            })
    };
}

/// Process a stream item from the parent agent, extracting text and tool calls.
fn process_parent_stream_item<R: Clone + std::fmt::Debug>(
    item: Result<MultiTurnStreamItem<R>, rig::agent::StreamingError>,
    event_tx: &mpsc::UnboundedSender<AgenticReviewEvent>,
    annotate_calls: &mut Vec<AnnotateFileArgs>,
) {
    match item {
        Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => match content {
            StreamedAssistantContent::Text(text) => {
                let _ = event_tx.send(AgenticReviewEvent::StreamToken(text.text));
            }
            StreamedAssistantContent::ToolCall {
                tool_call,
                internal_call_id: _,
            } => {
                if tool_call.function.name == "annotate_file" {
                    if let Ok(args) =
                        serde_json::from_value::<AnnotateFileArgs>(tool_call.function.arguments)
                    {
                        let _ = event_tx.send(AgenticReviewEvent::StreamToken(format!(
                            "\n\n> **{}** [{}|{}]: {}\n\n",
                            args.file_path, args.category, args.severity, args.concern
                        )));
                        annotate_calls.push(args);
                    }
                }
            }
            _ => {}
        },
        Ok(_) => {}
        Err(e) => {
            let _ = event_tx.send(AgenticReviewEvent::StreamToken(format!(
                "\n\nStream error: {}\n",
                e
            )));
        }
    }
}

// ================================================================
// Pipeline
// ================================================================

async fn run_pipeline(
    review_text: String,
    deltas: Vec<FileDelta>,
    config: AgenticReviewConfig,
    api_keys: Option<ApiKeysConfig>,
    repo_path: PathBuf,
    event_tx: &mpsc::UnboundedSender<AgenticReviewEvent>,
    kill_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let parent_provider = AiProvider::from_str(config.resolved_parent_provider())?;
    let parent_api_key =
        ai_client::resolve_api_key(&parent_provider, &api_keys).ok_or_else(|| {
            anyhow::anyhow!("Missing API key for {}", config.resolved_parent_provider())
        })?;
    let child_provider = AiProvider::from_str(config.resolved_child_provider())?;
    let child_api_key =
        ai_client::resolve_api_key(&child_provider, &api_keys).ok_or_else(|| {
            anyhow::anyhow!("Missing API key for {}", config.resolved_child_provider())
        })?;

    // Only override base_url if user explicitly configured one
    let base_url_override = ai_client::resolve_base_url_override(&config);
    let max_turns = config.max_agent_turns;

    let diff_text = render_diff_text(&deltas);
    let parent_user =
        format!("## Diff\n\n```diff\n{diff_text}```\n\n## User's Review\n\n{review_text}");

    // --- Phase 1: Parent agent streams review + tool calls ---
    let mut annotate_calls: Vec<AnnotateFileArgs> = Vec::new();

    // Dispatch based on provider type to avoid dyn issues
    match parent_provider {
        AiProvider::Anthropic => {
            let builder = rig::providers::anthropic::Client::builder().api_key(&parent_api_key);
            let client = maybe_base_url!(builder, base_url_override)
                .build()
                .map_err(|e| anyhow::anyhow!("Anthropic client error: {}", e))?;

            let agent = with_shared_tools!(
                client
                    .agent(&config.parent_model)
                    .preamble(PARENT_SYSTEM_PROMPT),
                repo_path
            )
            .tool(AnnotateFileTool)
            .default_max_turns(max_turns)
            .build();

            let stream = tokio::select! {
                s = agent.stream_prompt(&parent_user) => s,
                _ = kill_rx => return Ok(()),
            };

            let mut stream = std::pin::pin!(stream);
            while let Some(item) = stream.next().await {
                process_parent_stream_item(item, event_tx, &mut annotate_calls);
            }
        }
        AiProvider::OpenAI | AiProvider::Moonshot => {
            let builder = rig::providers::openai::Client::builder().api_key(&parent_api_key);
            let client = maybe_base_url!(builder, base_url_override)
                .build()
                .map_err(|e| anyhow::anyhow!("OpenAI client error: {}", e))?;

            let agent = with_shared_tools!(
                client
                    .agent(&config.parent_model)
                    .preamble(PARENT_SYSTEM_PROMPT),
                repo_path
            )
            .tool(AnnotateFileTool)
            .default_max_turns(max_turns)
            .build();

            let stream = tokio::select! {
                s = agent.stream_prompt(&parent_user) => s,
                _ = kill_rx => return Ok(()),
            };

            let mut stream = std::pin::pin!(stream);
            while let Some(item) = stream.next().await {
                process_parent_stream_item(item, event_tx, &mut annotate_calls);
            }
        }
    }

    if annotate_calls.is_empty() {
        let _ = event_tx.send(AgenticReviewEvent::Complete(vec![]));
        return Ok(());
    }

    // --- Phase 2: Child agents ---
    let total = annotate_calls.len();
    let _ = event_tx.send(AgenticReviewEvent::ChildProgress(0, total));

    let delta_map: std::collections::HashMap<String, &FileDelta> = deltas
        .iter()
        .map(|d| (d.path.to_string_lossy().to_string(), d))
        .collect();

    let mut annotations = Vec::new();
    let mut done = 0usize;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(5));
    let mut join_set = tokio::task::JoinSet::new();

    for call in annotate_calls {
        let sem = semaphore.clone();
        let child_api_key = child_api_key.clone();
        let child_base_url = base_url_override.clone();
        let child_model_name = config.child_model.clone();
        let repo_path = repo_path.clone();
        let file_diff = delta_map
            .get(&call.file_path)
            .map(|d| render_file_diff(d))
            .unwrap_or_default();
        let sink = Arc::new(Mutex::new(Vec::<CreateAnnotationArgs>::new()));

        let task_sink = sink.clone();
        join_set.spawn(async move {
            let _permit = sem.acquire().await;

            let child_user = format!(
                "## File Diff\n\n```diff\n{file_diff}```\n\n## Concern\n\n{}\n\nCategory: {}\nSeverity: {}",
                call.concern, call.category, call.severity
            );

            // Build and run child agent based on provider
            let result: Result<String> = match child_provider {
                AiProvider::Anthropic => {
                    let builder = rig::providers::anthropic::Client::builder()
                        .api_key(&child_api_key);
                    match maybe_base_url!(builder, child_base_url).build() {
                        Ok(client) => {
                            let agent = with_shared_tools!(
                                client.agent(&child_model_name).preamble(CHILD_SYSTEM_PROMPT),
                                repo_path
                            )
                            .tool(CreateAnnotationTool { sink: task_sink.clone() })
                            .default_max_turns(max_turns)
                            .build();
                            agent.prompt(&child_user).await.map_err(|e| anyhow::anyhow!("{}", e))
                        }
                        Err(e) => Err(anyhow::anyhow!("{}", e)),
                    }
                }
                AiProvider::OpenAI | AiProvider::Moonshot => {
                    let builder = rig::providers::openai::Client::builder()
                        .api_key(&child_api_key);
                    match maybe_base_url!(builder, child_base_url).build() {
                        Ok(client) => {
                            let agent = with_shared_tools!(
                                client.agent(&child_model_name).preamble(CHILD_SYSTEM_PROMPT),
                                repo_path
                            )
                            .tool(CreateAnnotationTool { sink: task_sink.clone() })
                            .default_max_turns(max_turns)
                            .build();
                            agent.prompt(&child_user).await.map_err(|e| anyhow::anyhow!("{}", e))
                        }
                        Err(e) => Err(anyhow::anyhow!("{}", e)),
                    }
                }
            };

            (call, result, sink)
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        done += 1;
        match join_result {
            Ok((call, Ok(_response), sink)) => {
                let captured: Vec<CreateAnnotationArgs> = sink
                    .lock()
                    .map(|mut v| std::mem::take(&mut *v))
                    .unwrap_or_default();

                if captured.is_empty() {
                    // Fallback: child didn't call create_annotation, use parent's data
                    let ann = Annotation {
                        anchor: LineAnchor {
                            file_path: call.file_path.clone(),
                            old_range: None,
                            new_range: None,
                        },
                        comment: call.concern.clone(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                        category: parse_category(&call.category),
                        severity: parse_severity(&call.severity),
                    };
                    annotations.push(ann);
                } else {
                    for args in captured {
                        let ann = Annotation {
                            anchor: LineAnchor {
                                file_path: args.file_path,
                                old_range: match (args.old_line_start, args.old_line_end) {
                                    (Some(s), Some(e)) => Some((s, e)),
                                    _ => None,
                                },
                                new_range: match (args.new_line_start, args.new_line_end) {
                                    (Some(s), Some(e)) => Some((s, e)),
                                    _ => None,
                                },
                            },
                            comment: args.comment,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            category: parse_category(&args.category),
                            severity: parse_severity(&args.severity),
                        };
                        annotations.push(ann);
                    }
                }
                let _ = event_tx.send(AgenticReviewEvent::ChildProgress(done, total));
            }
            Ok((call, Err(e), _sink)) => {
                let _ = event_tx.send(AgenticReviewEvent::StreamToken(format!(
                    "\n> Sub-agent error for {}: {}\n",
                    call.file_path, e
                )));
                let _ = event_tx.send(AgenticReviewEvent::ChildProgress(done, total));
            }
            Err(e) => {
                let _ = event_tx.send(AgenticReviewEvent::StreamToken(format!(
                    "\n> Sub-agent join error: {}\n",
                    e
                )));
                let _ = event_tx.send(AgenticReviewEvent::ChildProgress(done, total));
            }
        }
    }

    let _ = event_tx.send(AgenticReviewEvent::Complete(annotations));
    Ok(())
}

// ================================================================
// Prompts
// ================================================================

const PARENT_SYSTEM_PROMPT: &str = r#"You are a code review assistant. Read the diff and the user's feedback, then write a thorough, conversational review.

As you identify concerns, use the `annotate_file` tool to flag each one for detailed annotation by a sub-agent. Continue your review after each tool call.

You also have access to `list_files`, `read_file`, and `search` tools to explore the repository for additional context when needed.

Write naturally — your text output is what the user reads in the review panel. The tool calls create precise line-level annotations in the background.

Guidelines:
- Address each point in the user's feedback
- Identify concrete concerns with specific file paths from the diff
- Use annotate_file for each concern, with an appropriate category (Bug, Style, Performance, Security, Suggestion, Question, Nitpick) and severity (Critical, Major, Minor, Info)
- If the user's feedback doesn't relate to specific code, explain why and return without annotations
- Be concise but thorough"#;

const CHILD_SYSTEM_PROMPT: &str = r#"You are a precise code annotation assistant. Given a file's diff and a specific concern, identify the exact line range(s) that the concern applies to and create an annotation.

You have access to `list_files`, `read_file`, and `search` tools to explore the repository for additional context.

Once you've identified the exact lines, call `create_annotation` with:
- file_path: the exact file path
- new_line_start/new_line_end for additions or modified lines (lines with + prefix)
- old_line_start/old_line_end for deletions (lines with - prefix)
- Set the unused range to null
- category and severity from the parent's suggestion
- comment: an actionable, specific annotation

The line numbers must come from the diff provided."#;
