/// Complexity analysis for diff hunks using heuristic text patterns.
///
/// This module provides complexity scoring for individual hunks and files
/// to help reviewers prioritize their attention on the most important changes.
/// Complexity score for a hunk or file (0-10 scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplexityScore {
    pub score: u8,           // 0-10
    pub label: &'static str, // "Low", "Med", "High", "Critical"
}

impl ComplexityScore {
    pub fn new(score: u8) -> Self {
        let label = match score {
            0..=2 => "Low",
            3..=4 => "Med",
            5..=7 => "High",
            _ => "Critical",
        };
        Self {
            score: score.min(10),
            label,
        }
    }
}

/// Hunk-level complexity analysis result.
#[derive(Debug, Clone)]
pub struct HunkComplexity {
    pub score: ComplexityScore,
    pub factors: Vec<ComplexityFactor>,
}

#[derive(Debug, Clone)]
pub enum ComplexityFactor {
    NestingDepthIncrease(u8),
    NewControlFlow(u8),   // if/match/loop/for/while added
    LargeHunkSize(usize), // lines changed
    NewUnsafeBlock,
    NewUnwrapCall(u8),
    ErrorHandlingChange,
    PublicApiChange,
    NewDependency,
}

impl ComplexityFactor {
    pub fn label(&self) -> &'static str {
        match self {
            ComplexityFactor::NestingDepthIncrease(_) => "nesting",
            ComplexityFactor::NewControlFlow(_) => "control_flow",
            ComplexityFactor::LargeHunkSize(_) => "large_hunk",
            ComplexityFactor::NewUnsafeBlock => "unsafe",
            ComplexityFactor::NewUnwrapCall(_) => "unwrap",
            ComplexityFactor::ErrorHandlingChange => "error_handling",
            ComplexityFactor::PublicApiChange => "public_api",
            ComplexityFactor::NewDependency => "dependency",
        }
    }

    pub fn points(&self) -> u8 {
        match self {
            ComplexityFactor::NestingDepthIncrease(n) => *n,
            ComplexityFactor::NewControlFlow(n) => *n,
            ComplexityFactor::LargeHunkSize(size) => {
                if *size > 100 {
                    4
                } else if *size > 30 {
                    2
                } else {
                    0
                }
            }
            ComplexityFactor::NewUnsafeBlock => 3,
            ComplexityFactor::NewUnwrapCall(n) => *n,
            ComplexityFactor::ErrorHandlingChange => 1,
            ComplexityFactor::PublicApiChange => 2,
            ComplexityFactor::NewDependency => 1,
        }
    }
}

/// Analyze a hunk's added lines for complexity signals.
pub fn analyze_hunk(
    added_lines: &[&str],
    removed_lines: &[&str],
    file_path: &str,
) -> HunkComplexity {
    let mut factors = Vec::new();
    let mut total_points = 0u8;

    // Large hunk size
    let hunk_size = added_lines.len() + removed_lines.len();
    if hunk_size > 30 {
        let factor = ComplexityFactor::LargeHunkSize(hunk_size);
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    // Analyze added lines for complexity patterns
    let mut nesting_increase = 0u8;
    let mut control_flow_count = 0u8;
    let mut unwrap_count = 0u8;
    let mut has_unsafe = false;
    let mut has_error_handling = false;
    let mut has_public_api = false;
    let mut has_dependency = false;

    // Calculate average indentation increase
    let added_indent = calculate_average_indentation(added_lines);
    let removed_indent = calculate_average_indentation(removed_lines);
    if added_indent > removed_indent {
        nesting_increase = ((added_indent - removed_indent) / 4.0).ceil() as u8;
    }

    for line in added_lines {
        let trimmed = line.trim();

        // Control flow keywords
        if contains_control_flow(trimmed) {
            control_flow_count = control_flow_count.saturating_add(1);
        }

        // Unsafe blocks (Rust-specific)
        if is_rust_file(file_path)
            && (trimmed.contains("unsafe {") || trimmed.contains("unsafe fn"))
        {
            has_unsafe = true;
        }

        // Unwrap calls (Rust-specific)
        if is_rust_file(file_path) && trimmed.contains(".unwrap()") {
            // Don't count if it was already in removed lines
            if !removed_lines.iter().any(|r| r.trim().contains(".unwrap()")) {
                unwrap_count = unwrap_count.saturating_add(1);
            }
        }

        // Error handling patterns
        if contains_error_handling(trimmed) {
            has_error_handling = true;
        }

        // Public API changes
        if contains_public_api(trimmed) {
            has_public_api = true;
        }

        // Dependency changes (Cargo.toml)
        if file_path.ends_with("Cargo.toml") && contains_version_specifier(trimmed) {
            has_dependency = true;
        }
    }

    // Add factors and accumulate points
    if nesting_increase > 0 {
        let factor = ComplexityFactor::NestingDepthIncrease(nesting_increase);
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    if control_flow_count > 0 {
        let factor = ComplexityFactor::NewControlFlow(control_flow_count);
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    if has_unsafe {
        let factor = ComplexityFactor::NewUnsafeBlock;
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    if unwrap_count > 0 {
        let factor = ComplexityFactor::NewUnwrapCall(unwrap_count);
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    if has_error_handling {
        let factor = ComplexityFactor::ErrorHandlingChange;
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    if has_public_api {
        let factor = ComplexityFactor::PublicApiChange;
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    if has_dependency {
        let factor = ComplexityFactor::NewDependency;
        total_points = total_points.saturating_add(factor.points());
        factors.push(factor);
    }

    HunkComplexity {
        score: ComplexityScore::new(total_points),
        factors,
    }
}

/// Aggregate hunk scores into a file-level score.
pub fn file_complexity(hunks: &[HunkComplexity]) -> ComplexityScore {
    if hunks.is_empty() {
        return ComplexityScore::new(0);
    }

    // Use the maximum hunk score as the file score
    let max_score = hunks.iter().map(|h| h.score.score).max().unwrap_or(0);
    ComplexityScore::new(max_score)
}

/// Calculate average indentation level for a set of lines.
fn calculate_average_indentation(lines: &[&str]) -> f32 {
    if lines.is_empty() {
        return 0.0;
    }

    let total_indent: usize = lines
        .iter()
        .map(|line| {
            line.chars()
                .take_while(|&c| c == ' ' || c == '\t')
                .map(|c| if c == '\t' { 4 } else { 1 })
                .sum::<usize>()
        })
        .sum();

    total_indent as f32 / lines.len() as f32
}

/// Check if a line contains control flow keywords.
fn contains_control_flow(line: &str) -> bool {
    // Language-agnostic control flow patterns
    let keywords = [
        "if ", "else ", "elif ", "elsif ", "match ", "switch ", "case ", "for ", "while ", "loop ",
        "do ", "try ", "catch ", "except ",
    ];

    keywords.iter().any(|&keyword| {
        line.contains(keyword) &&
        // Avoid false positives in comments and strings
        !line.trim_start().starts_with("//") &&
        !line.trim_start().starts_with("#") &&
        !line.trim_start().starts_with("*")
    })
}

/// Check if a line contains error handling patterns.
fn contains_error_handling(line: &str) -> bool {
    line.contains("Result")
        || line.contains("Error")
        || line.contains("?")
        || line.contains("try!")
        || line.contains("expect(")
        || line.contains("panic!(")
}

/// Check if a line contains public API declarations.
fn contains_public_api(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("pub fn ") ||
    trimmed.starts_with("pub struct ") ||
    trimmed.starts_with("pub enum ") ||
    trimmed.starts_with("pub trait ") ||
    trimmed.starts_with("pub const ") ||
    trimmed.starts_with("pub static ") ||
    trimmed.starts_with("pub mod ") ||
    trimmed.starts_with("pub use ") ||
    trimmed.starts_with("export ") || // JavaScript/TypeScript
    trimmed.starts_with("public ") || // Java/C#
    trimmed.contains("__all__") // Python
}

/// Check if a line contains a version specifier (for dependency detection).
fn contains_version_specifier(line: &str) -> bool {
    // Pattern for version numbers like "1.2.3", "^0.5", "~1.0"
    line.contains("version")
        && (line.contains("\"") || line.contains("'"))
        && (line.contains(char::is_numeric) || line.contains("^") || line.contains("~"))
}

/// Check if a file is a Rust source file.
fn is_rust_file(path: &str) -> bool {
    path.ends_with(".rs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_score_labels() {
        assert_eq!(ComplexityScore::new(0).label, "Low");
        assert_eq!(ComplexityScore::new(2).label, "Low");
        assert_eq!(ComplexityScore::new(3).label, "Med");
        assert_eq!(ComplexityScore::new(4).label, "Med");
        assert_eq!(ComplexityScore::new(5).label, "High");
        assert_eq!(ComplexityScore::new(7).label, "High");
        assert_eq!(ComplexityScore::new(8).label, "Critical");
        assert_eq!(ComplexityScore::new(15).label, "Critical"); // clamped to 10
    }

    #[test]
    fn test_simple_hunk_analysis() {
        let added = vec!["    println!(\"hello\");"];
        let removed = vec![];
        let result = analyze_hunk(&added, &removed, "test.rs");

        // Should be low complexity - just a simple print statement
        assert_eq!(result.score.label, "Low");
    }

    #[test]
    fn test_control_flow_detection() {
        let added = vec![
            "if condition {",
            "    for item in items {",
            "        match item {",
        ];
        let removed = vec![];
        let result = analyze_hunk(&added, &removed, "test.rs");

        // Should detect control flow
        assert!(result
            .factors
            .iter()
            .any(|f| matches!(f, ComplexityFactor::NewControlFlow(_))));
        assert!(result.score.score > 0);
    }

    #[test]
    fn test_unsafe_detection() {
        let added = vec!["unsafe { ptr.read() }"];
        let removed = vec![];
        let result = analyze_hunk(&added, &removed, "test.rs");

        assert!(result
            .factors
            .iter()
            .any(|f| matches!(f, ComplexityFactor::NewUnsafeBlock)));
        assert!(result.score.score >= 3);
    }

    #[test]
    fn test_large_hunk_detection() {
        let added: Vec<&str> = (0..50).map(|_| "    some_code();").collect();
        let removed = vec![];
        let result = analyze_hunk(&added, &removed, "test.rs");

        assert!(result
            .factors
            .iter()
            .any(|f| matches!(f, ComplexityFactor::LargeHunkSize(_))));
    }

    #[test]
    fn test_public_api_detection() {
        let added = vec!["pub fn new_function() -> Result<(), Error> {"];
        let removed = vec![];
        let result = analyze_hunk(&added, &removed, "test.rs");

        assert!(result
            .factors
            .iter()
            .any(|f| matches!(f, ComplexityFactor::PublicApiChange)));
        assert!(result
            .factors
            .iter()
            .any(|f| matches!(f, ComplexityFactor::ErrorHandlingChange)));
    }

    #[test]
    fn test_file_complexity_aggregation() {
        let hunks = vec![
            HunkComplexity {
                score: ComplexityScore::new(2),
                factors: vec![],
            },
            HunkComplexity {
                score: ComplexityScore::new(5),
                factors: vec![],
            },
            HunkComplexity {
                score: ComplexityScore::new(1),
                factors: vec![],
            },
        ];

        let file_score = file_complexity(&hunks);
        assert_eq!(file_score.score, 5); // Should use max score
        assert_eq!(file_score.label, "High");
    }

    #[test]
    fn test_nesting_calculation() {
        let added = vec![
            "        if condition {",      // 8 spaces
            "            do_something();", // 12 spaces
        ];
        let removed = vec![
            "    simple_call();", // 4 spaces
        ];
        let result = analyze_hunk(&added, &removed, "test.rs");

        // Should detect nesting increase
        assert!(result
            .factors
            .iter()
            .any(|f| matches!(f, ComplexityFactor::NestingDepthIncrease(_))));
    }
}
