use crate::git::types::{ComparisonTarget, FileDelta};
use crate::state::diff_state::DiffOptions;

#[derive(Debug, Clone)]
pub struct DiffRequest {
    pub generation: u64,
    pub target: ComparisonTarget,
    pub options: DiffOptions,
}

#[derive(Debug)]
pub struct DiffResult {
    pub generation: u64,
    pub deltas: Result<Vec<FileDelta>, String>,
}
