use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Untracked,
}

impl FileStatus {
    pub fn label(&self) -> &'static str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Deleted => "D",
            FileStatus::Modified => "M",
            FileStatus::Renamed => "R",
            FileStatus::Untracked => "?",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineOrigin {
    Context,
    Addition,
    Deletion,
    #[allow(dead_code)]
    HunkHeader,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub origin: DiffLineOrigin,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Hunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct FileDelta {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
    pub additions: usize,
    pub deletions: usize,
    pub binary: bool,
}

#[derive(Debug, Clone)]
pub enum ComparisonTarget {
    HeadVsWorkdir,
    Branch(String),
    Commit(git2::Oid),
    #[allow(dead_code)]
    Ref(String),
}
