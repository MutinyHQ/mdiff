use crate::git::types::{FileDelta, FileStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Churn,
    Additions,
    Name,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Churn => SortMode::Additions,
            SortMode::Additions => SortMode::Name,
            SortMode::Name => SortMode::Churn,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Churn => "by churn",
            SortMode::Additions => "by additions",
            SortMode::Name => "by name",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub path: String,
    #[allow(dead_code)]
    pub old_path: Option<String>,
    pub additions: usize,
    pub deletions: usize,
    pub status: FileStatus,
    pub binary: bool,
    pub delta_index: usize,
}

impl FileStats {
    pub fn churn(&self) -> usize {
        self.additions + self.deletions
    }
}

#[derive(Debug, Clone)]
pub struct DiffSummary {
    pub total_files: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub files_added: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
    pub files_renamed: usize,
    pub by_extension: Vec<(String, usize)>,
    pub file_stats: Vec<FileStats>,
}

impl DiffSummary {
    pub fn from_deltas(deltas: &[FileDelta]) -> Self {
        let mut total_additions = 0usize;
        let mut total_deletions = 0usize;
        let mut files_added = 0usize;
        let mut files_modified = 0usize;
        let mut files_deleted = 0usize;
        let mut files_renamed = 0usize;
        let mut ext_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut file_stats = Vec::with_capacity(deltas.len());

        for (i, delta) in deltas.iter().enumerate() {
            total_additions += delta.additions;
            total_deletions += delta.deletions;

            match delta.status {
                FileStatus::Added => files_added += 1,
                FileStatus::Modified => files_modified += 1,
                FileStatus::Deleted => files_deleted += 1,
                FileStatus::Renamed => files_renamed += 1,
                FileStatus::Untracked => files_added += 1,
            }

            let ext = delta
                .path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_else(|| "other".to_string());
            *ext_counts.entry(ext).or_insert(0) += 1;

            file_stats.push(FileStats {
                path: delta.path.to_string_lossy().to_string(),
                old_path: delta
                    .old_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                additions: delta.additions,
                deletions: delta.deletions,
                status: delta.status.clone(),
                binary: delta.binary,
                delta_index: i,
            });
        }

        file_stats.sort_by_key(|f| std::cmp::Reverse(f.churn()));

        let mut by_extension: Vec<(String, usize)> = ext_counts.into_iter().collect();
        by_extension.sort_by(|a, b| b.1.cmp(&a.1));

        DiffSummary {
            total_files: deltas.len(),
            total_additions,
            total_deletions,
            files_added,
            files_modified,
            files_deleted,
            files_renamed,
            by_extension,
            file_stats,
        }
    }

    pub fn sorted_file_stats(&self, mode: SortMode) -> Vec<FileStats> {
        let mut stats = self.file_stats.clone();
        match mode {
            SortMode::Churn => stats.sort_by_key(|f| std::cmp::Reverse(f.churn())),
            SortMode::Additions => stats.sort_by_key(|f| std::cmp::Reverse(f.additions)),
            SortMode::Name => stats.sort_by(|a, b| a.path.cmp(&b.path)),
        }
        stats
    }
}

#[derive(Debug)]
pub struct StatsState {
    pub open: bool,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub sort_mode: SortMode,
    pub diff_summary: Option<DiffSummary>,
}

impl Default for StatsState {
    fn default() -> Self {
        Self {
            open: false,
            scroll_offset: 0,
            selected_index: 0,
            sort_mode: SortMode::Churn,
            diff_summary: None,
        }
    }
}
