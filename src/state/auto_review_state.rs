use crate::state::annotation_state::{Annotation, AnnotationCategory, AnnotationSeverity};

/// Status of an individual review finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingStatus {
    Pending,
    Accepted,
    Dismissed,
}

impl FindingStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Accepted => "Accepted",
            Self::Dismissed => "Dismissed",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "●",
            Self::Accepted => "✓",
            Self::Dismissed => "✗",
        }
    }
}

/// Filter mode for the findings list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingFilter {
    All,
    Pending,
    Accepted,
    Dismissed,
}

impl FindingFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Pending => "Pending",
            Self::Accepted => "Accepted",
            Self::Dismissed => "Dismissed",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::All => Self::Pending,
            Self::Pending => Self::Accepted,
            Self::Accepted => Self::Dismissed,
            Self::Dismissed => Self::All,
        }
    }
}

/// A single AI-generated review finding.
#[derive(Debug, Clone)]
pub struct ReviewFinding {
    pub file_path: String,
    pub old_range: Option<(u32, u32)>,
    pub new_range: Option<(u32, u32)>,
    pub comment: String,
    pub category: AnnotationCategory,
    pub severity: AnnotationSeverity,
    pub status: FindingStatus,
}

impl ReviewFinding {
    /// Representative line for display/sorting, preferring new-file.
    pub fn sort_line(&self) -> u32 {
        self.new_range
            .map(|(s, _)| s)
            .or(self.old_range.map(|(s, _)| s))
            .unwrap_or(0)
    }

    /// Format a human-readable range string.
    pub fn range_text(&self) -> String {
        match (self.old_range, self.new_range) {
            (_, Some((s, e))) if s == e => format!("L{s}"),
            (_, Some((s, e))) => format!("L{s}-{e}"),
            (Some((s, e)), None) if s == e => format!("old L{s}"),
            (Some((s, e)), None) => format!("old L{s}-{e}"),
            (None, None) => "file".to_string(),
        }
    }

    /// Convert this finding into an Annotation for the persistent annotation system.
    pub fn to_annotation(&self) -> Annotation {
        Annotation {
            anchor: crate::state::annotation_state::LineAnchor {
                file_path: self.file_path.clone(),
                old_range: self.old_range,
                new_range: self.new_range,
            },
            comment: self.comment.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            category: self.category,
            severity: self.severity,
        }
    }
}

/// State for the auto-review findings panel.
#[derive(Debug)]
pub struct AutoReviewState {
    pub findings: Vec<ReviewFinding>,
    pub selected: usize,
    pub filter: FindingFilter,
    pub panel_open: bool,
    pub scroll_offset: usize,
}

impl Default for AutoReviewState {
    fn default() -> Self {
        Self {
            findings: Vec::new(),
            selected: 0,
            filter: FindingFilter::All,
            panel_open: false,
            scroll_offset: 0,
        }
    }
}

impl AutoReviewState {
    /// Populate findings from completed annotations (does not add them to AnnotationState).
    pub fn populate(&mut self, annotations: &[Annotation]) {
        self.findings = annotations
            .iter()
            .map(|ann| ReviewFinding {
                file_path: ann.anchor.file_path.clone(),
                old_range: ann.anchor.old_range,
                new_range: ann.anchor.new_range,
                comment: ann.comment.clone(),
                category: ann.category,
                severity: ann.severity,
                status: FindingStatus::Pending,
            })
            .collect();
        self.findings
            .sort_by(|a, b| (&a.file_path, a.sort_line()).cmp(&(&b.file_path, b.sort_line())));
        self.selected = 0;
        self.scroll_offset = 0;
        self.panel_open = true;
        self.filter = FindingFilter::All;
    }

    /// Indices of findings that match the current filter.
    pub fn filtered_indices(&self) -> Vec<usize> {
        self.findings
            .iter()
            .enumerate()
            .filter(|(_, f)| match self.filter {
                FindingFilter::All => true,
                FindingFilter::Pending => f.status == FindingStatus::Pending,
                FindingFilter::Accepted => f.status == FindingStatus::Accepted,
                FindingFilter::Dismissed => f.status == FindingStatus::Dismissed,
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Total and reviewed counts for header display.
    pub fn reviewed_count(&self) -> (usize, usize) {
        let total = self.findings.len();
        let reviewed = self
            .findings
            .iter()
            .filter(|f| f.status != FindingStatus::Pending)
            .count();
        (reviewed, total)
    }

    /// Move selection to next visible finding.
    pub fn select_next(&mut self) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            return;
        }
        if let Some(pos) = indices.iter().position(|&i| i == self.selected) {
            let next = (pos + 1).min(indices.len() - 1);
            self.selected = indices[next];
        } else if let Some(&first) = indices.first() {
            self.selected = first;
        }
    }

    /// Move selection to previous visible finding.
    pub fn select_prev(&mut self) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            return;
        }
        if let Some(pos) = indices.iter().position(|&i| i == self.selected) {
            let prev = pos.saturating_sub(1);
            self.selected = indices[prev];
        } else if let Some(&last) = indices.last() {
            self.selected = last;
        }
    }

    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::annotation_state::{Annotation, LineAnchor};

    fn sample_annotations() -> Vec<Annotation> {
        vec![
            Annotation {
                anchor: LineAnchor {
                    file_path: "src/main.rs".to_string(),
                    old_range: None,
                    new_range: Some((10, 15)),
                },
                comment: "Potential null deref".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                category: AnnotationCategory::Bug,
                severity: AnnotationSeverity::Critical,
            },
            Annotation {
                anchor: LineAnchor {
                    file_path: "src/lib.rs".to_string(),
                    old_range: Some((5, 5)),
                    new_range: Some((5, 8)),
                },
                comment: "Style nit".to_string(),
                created_at: "2026-01-01T00:00:01Z".to_string(),
                category: AnnotationCategory::Style,
                severity: AnnotationSeverity::Info,
            },
        ]
    }

    #[test]
    fn populate_creates_sorted_findings() {
        let mut state = AutoReviewState::default();
        state.populate(&sample_annotations());
        assert_eq!(state.findings.len(), 2);
        // lib.rs sorts before main.rs
        assert_eq!(state.findings[0].file_path, "src/lib.rs");
        assert_eq!(state.findings[1].file_path, "src/main.rs");
        assert!(state.panel_open);
    }

    #[test]
    fn filter_cycle() {
        let f = FindingFilter::All;
        assert_eq!(f.cycle(), FindingFilter::Pending);
        assert_eq!(f.cycle().cycle(), FindingFilter::Accepted);
        assert_eq!(f.cycle().cycle().cycle(), FindingFilter::Dismissed);
        assert_eq!(f.cycle().cycle().cycle().cycle(), FindingFilter::All);
    }

    #[test]
    fn reviewed_count_tracks_status() {
        let mut state = AutoReviewState::default();
        state.populate(&sample_annotations());
        assert_eq!(state.reviewed_count(), (0, 2));
        state.findings[0].status = FindingStatus::Accepted;
        assert_eq!(state.reviewed_count(), (1, 2));
        state.findings[1].status = FindingStatus::Dismissed;
        assert_eq!(state.reviewed_count(), (2, 2));
    }

    #[test]
    fn filtered_indices_respects_filter() {
        let mut state = AutoReviewState::default();
        state.populate(&sample_annotations());
        state.findings[0].status = FindingStatus::Accepted;

        state.filter = FindingFilter::Pending;
        assert_eq!(state.filtered_indices(), vec![1]);

        state.filter = FindingFilter::Accepted;
        assert_eq!(state.filtered_indices(), vec![0]);

        state.filter = FindingFilter::All;
        assert_eq!(state.filtered_indices(), vec![0, 1]);
    }

    #[test]
    fn navigation_wraps_within_filtered() {
        let mut state = AutoReviewState::default();
        state.populate(&sample_annotations());
        assert_eq!(state.selected, 0);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 1); // stays at last
        state.select_prev();
        assert_eq!(state.selected, 0);
        state.select_prev();
        assert_eq!(state.selected, 0); // stays at first
    }
}
