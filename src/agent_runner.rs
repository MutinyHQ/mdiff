use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// Events emitted by a running agent process.
#[derive(Debug)]
pub enum AgentEvent {
    /// A line of output (stdout or stderr).
    OutputLine(usize, String),
    /// Process exited with the given code.
    Done(usize, i32),
}

/// Manages a running agent subprocess.
/// Spawns via `sh -c` and streams stdout/stderr line by line.
pub struct AgentRunner {
    event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    kill_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl AgentRunner {
    /// Spawn an agent subprocess. Returns an AgentRunner that can be polled for events.
    pub fn spawn(run_id: usize, command: &str) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();

        let cmd = command.to_string();

        tokio::spawn(async move {
            let mut child = match Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = event_tx.send(AgentEvent::OutputLine(
                        run_id,
                        format!("Failed to spawn: {e}"),
                    ));
                    let _ = event_tx.send(AgentEvent::Done(run_id, 1));
                    return;
                }
            };

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let tx_out = event_tx.clone();
            let stdout_handle = tokio::spawn(async move {
                if let Some(stdout) = stdout {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if tx_out.send(AgentEvent::OutputLine(run_id, line)).is_err() {
                            break;
                        }
                    }
                }
            });

            let tx_err = event_tx.clone();
            let stderr_handle = tokio::spawn(async move {
                if let Some(stderr) = stderr {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if tx_err.send(AgentEvent::OutputLine(run_id, line)).is_err() {
                            break;
                        }
                    }
                }
            });

            // Wait for either completion or kill signal
            tokio::select! {
                status = child.wait() => {
                    let _ = stdout_handle.await;
                    let _ = stderr_handle.await;
                    let code = status.map(|s| s.code().unwrap_or(1)).unwrap_or(1);
                    let _ = event_tx.send(AgentEvent::Done(run_id, code));
                }
                _ = kill_rx => {
                    let _ = child.kill().await;
                    let _ = event_tx.send(AgentEvent::OutputLine(run_id, "[Process killed]".to_string()));
                    let _ = event_tx.send(AgentEvent::Done(run_id, 137));
                }
            }
        });

        Self {
            event_rx,
            kill_tx: Some(kill_tx),
        }
    }

    /// Non-blocking poll for events.
    pub fn try_recv(&mut self) -> Option<AgentEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Kill the running process.
    pub fn kill(&mut self) {
        if let Some(tx) = self.kill_tx.take() {
            let _ = tx.send(());
        }
    }
}
