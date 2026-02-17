use std::path::PathBuf;

use tokio::sync::mpsc;

use crate::git::{DiffEngine, RepoCache};

use super::channel::{DiffRequest, DiffResult};

pub struct DiffWorker {
    request_tx: mpsc::UnboundedSender<DiffRequest>,
    result_rx: mpsc::UnboundedReceiver<DiffResult>,
}

impl DiffWorker {
    pub fn new(repo_path: PathBuf) -> Self {
        let (request_tx, mut request_rx) = mpsc::unbounded_channel::<DiffRequest>();
        let (result_tx, result_rx) = mpsc::unbounded_channel::<DiffResult>();

        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let path = repo_path.clone();
                let tx = result_tx.clone();

                tokio::task::spawn_blocking(move || {
                    let result = match RepoCache::open(&path) {
                        Ok(repo) => {
                            match DiffEngine::compute_diff(
                                repo.repo(),
                                &request.target,
                                &request.options,
                            ) {
                                Ok(deltas) => DiffResult {
                                    generation: request.generation,
                                    deltas: Ok(deltas),
                                },
                                Err(e) => DiffResult {
                                    generation: request.generation,
                                    deltas: Err(e.to_string()),
                                },
                            }
                        }
                        Err(e) => DiffResult {
                            generation: request.generation,
                            deltas: Err(e.to_string()),
                        },
                    };
                    let _ = tx.send(result);
                });
            }
        });

        Self {
            request_tx,
            result_rx,
        }
    }

    pub fn request(&self, req: DiffRequest) {
        let _ = self.request_tx.send(req);
    }

    pub fn try_recv(&mut self) -> Option<DiffResult> {
        self.result_rx.try_recv().ok()
    }
}
