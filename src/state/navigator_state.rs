use crate::git::types::FileDelta;
use std::collections::{BTreeMap, HashMap, HashSet};

use super::TextBuffer;

#[derive(Debug)]
pub struct NavigatorEntry {
    pub display: String,
    pub path: String,
    pub delta_index: usize,
}

#[derive(Debug, Clone)]
pub enum TreeNodeKind {
    Directory {
        name: String,
        collapsed: bool,
        file_count: usize,
        total_additions: usize,
        total_deletions: usize,
    },
    File {
        entry_index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub kind: TreeNodeKind,
    pub depth: usize,
    pub path: String,
}

#[derive(Debug)]
pub struct NavigatorState {
    pub selected: usize,
    pub entries: Vec<NavigatorEntry>,
    pub filtered_indices: Vec<usize>,
    pub search_active: bool,
    pub search_query: TextBuffer,
    /// Saved selection index before search started (for cancel/restore).
    pre_search_selected: Option<usize>,
    pub tree_mode: bool,
    pub tree_nodes: Vec<TreeNode>,
    pub tree_selected: usize,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            entries: Vec::new(),
            filtered_indices: Vec::new(),
            search_active: false,
            search_query: TextBuffer::new(),
            pre_search_selected: None,
            tree_mode: false,
            tree_nodes: Vec::new(),
            tree_selected: 0,
        }
    }

    pub fn update_from_deltas(&mut self, deltas: &[FileDelta]) {
        let paths: Vec<String> = deltas
            .iter()
            .map(|d| d.path.to_string_lossy().to_string())
            .collect();
        let informative_paths = build_informative_path_displays(&paths);

        self.entries = deltas
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let path_str = paths[i].clone();
                let display = format!(
                    "{} [{}] +{} -{}",
                    informative_paths[i],
                    d.status.label(),
                    d.additions,
                    d.deletions
                );
                NavigatorEntry {
                    display,
                    path: path_str,
                    delta_index: i,
                }
            })
            .collect();

        self.refilter();

        if self.tree_mode {
            self.rebuild_tree();
        }
    }

    pub fn refilter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
        } else {
            let query_lower = self.search_query.text().to_lowercase();
            self.filtered_indices = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| fuzzy_match(&e.path.to_lowercase(), &query_lower))
                .map(|(i, _)| i)
                .collect();
        }

        // Clamp selection
        if !self.filtered_indices.is_empty() {
            self.selected = self.selected.min(self.filtered_indices.len() - 1);
        } else {
            self.selected = 0;
        }
    }

    pub fn visible_entries(&self) -> Vec<(usize, &NavigatorEntry)> {
        self.filtered_indices
            .iter()
            .map(|&i| (i, &self.entries[i]))
            .collect()
    }

    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_indices.len() - 1);
        }
    }

    pub fn selected_delta_index(&self) -> Option<usize> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&i| self.entries.get(i))
            .map(|e| e.delta_index)
    }

    pub fn start_search(&mut self) {
        self.pre_search_selected = Some(self.selected);
        self.search_active = true;
        self.search_query.clear();
    }

    /// Confirm search (Enter): resolve the currently selected entry, then
    /// clear the query and refilter so all entries are visible again, keeping
    /// focus on the entry that was selected in the filtered list.
    pub fn confirm_search(&mut self) {
        let target_delta_index = self.selected_delta_index();
        self.search_active = false;
        self.search_query.clear();
        self.refilter();
        // Find the entry with the same delta_index in the now-unfiltered list
        if let Some(delta_idx) = target_delta_index {
            if let Some(pos) = self
                .filtered_indices
                .iter()
                .position(|&i| self.entries[i].delta_index == delta_idx)
            {
                self.selected = pos;
            }
        }
        self.pre_search_selected = None;
    }

    /// Cancel search (Esc): restore the selection from before search started.
    pub fn cancel_search(&mut self) {
        let restore = self.pre_search_selected.take();
        self.search_active = false;
        self.search_query.clear();
        self.refilter();
        if let Some(prev) = restore {
            self.selected = prev.min(self.filtered_indices.len().saturating_sub(1));
        }
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.insert_char(c);
        self.selected = 0;
        self.refilter();
    }

    pub fn search_pop(&mut self) {
        self.search_query.delete_back();
        self.selected = 0;
        self.refilter();
    }

    // --- Tree mode ---

    pub fn toggle_tree_mode(&mut self) {
        self.tree_mode = !self.tree_mode;
        if self.tree_mode {
            self.rebuild_tree();
            self.tree_selected = 0;
        } else {
            self.map_tree_selection_to_flat();
        }
    }

    pub fn rebuild_tree(&mut self) {
        let entries = &self.entries;
        if entries.is_empty() {
            self.tree_nodes.clear();
            return;
        }

        // Collect per-directory aggregates and file membership using a trie approach.
        // dir_path -> (file_count, additions, deletions, collapsed_state)
        let mut dir_stats: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
        // Preserve collapsed state from previous tree_nodes
        let mut prev_collapsed: HashMap<String, bool> = HashMap::new();
        for node in &self.tree_nodes {
            if let TreeNodeKind::Directory {
                collapsed,
                ref name,
                ..
            } = node.kind
            {
                let _ = name;
                prev_collapsed.insert(node.path.clone(), collapsed);
            }
        }

        for (i, entry) in entries.iter().enumerate() {
            let _ = i;
            let parts: Vec<&str> = entry.path.rsplitn(2, '/').collect();
            let dir = if parts.len() == 2 { parts[1] } else { "" };

            // Build stats for this directory and all ancestor directories
            let components: Vec<&str> = if dir.is_empty() {
                vec![]
            } else {
                dir.split('/').collect()
            };
            for depth in 0..=components.len() {
                let ancestor = if depth == 0 {
                    String::new()
                } else {
                    components[..depth].join("/")
                };
                // Only add real directories (not root)
                if !ancestor.is_empty() {
                    dir_stats.entry(ancestor).or_insert((0, 0, 0));
                }
            }

            // Add file stats to its immediate parent directory
            let immediate_dir = if dir.is_empty() {
                String::new()
            } else {
                dir.to_string()
            };
            if !immediate_dir.is_empty() {
                let stat = dir_stats.entry(immediate_dir).or_insert((0, 0, 0));
                stat.0 += 1;
            }
        }

        // Compute aggregate stats by walking all entries for each dir (including nested)
        let mut dir_agg: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
        for entry in entries.iter() {
            let parts: Vec<&str> = entry.path.rsplitn(2, '/').collect();
            let dir = if parts.len() == 2 { parts[1] } else { "" };
            let components: Vec<&str> = if dir.is_empty() {
                vec![]
            } else {
                dir.split('/').collect()
            };

            let delta = &entries[entry.delta_index];
            let (additions, deletions) = parse_additions_deletions(&delta.display);

            for depth in 1..=components.len() {
                let ancestor = components[..depth].join("/");
                let stat = dir_agg.entry(ancestor).or_insert((0, 0, 0));
                stat.0 += 1;
                stat.1 += additions;
                stat.2 += deletions;
            }
        }

        // Build a map: dir_path -> Vec<entry_index> (files directly in that dir)
        let mut dir_files: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        // Root files (no directory)
        let mut root_files: Vec<usize> = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            let parts: Vec<&str> = entry.path.rsplitn(2, '/').collect();
            if parts.len() == 2 {
                dir_files.entry(parts[1].to_string()).or_default().push(i);
            } else {
                root_files.push(i);
            }
        }

        // Sort files within each directory alphabetically
        for files in dir_files.values_mut() {
            files.sort_by(|a, b| entries[*a].path.cmp(&entries[*b].path));
        }
        root_files.sort_by(|a, b| entries[*a].path.cmp(&entries[*b].path));

        // Flatten: We need to output directories in sorted order, with their children.
        // Only output directories that directly contain files (leaf dirs) or have subdirs that do.
        // Use a recursive approach via iteration.

        let all_dirs: Vec<String> = dir_agg.keys().cloned().collect();

        // Build parent->children map for directories
        let mut dir_children: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for d in &all_dirs {
            let parts: Vec<&str> = d.rsplitn(2, '/').collect();
            let parent = if parts.len() == 2 {
                parts[1].to_string()
            } else {
                String::new()
            };
            dir_children.entry(parent).or_default().push(d.clone());
        }
        // Sort children
        for children in dir_children.values_mut() {
            children.sort();
        }

        self.tree_nodes.clear();

        #[allow(clippy::too_many_arguments)]
        fn flatten_tree(
            parent: &str,
            depth: usize,
            dir_children: &BTreeMap<String, Vec<String>>,
            dir_files: &BTreeMap<String, Vec<usize>>,
            dir_agg: &BTreeMap<String, (usize, usize, usize)>,
            prev_collapsed: &HashMap<String, bool>,
            entries: &[NavigatorEntry],
            result: &mut Vec<TreeNode>,
        ) {
            // First, output sub-directories of this parent
            if let Some(children) = dir_children.get(parent) {
                for child_dir in children {
                    let dir_depth = child_dir.matches('/').count();
                    // Only output if this is a direct child (depth == parent_depth + 1)
                    let parent_depth = if parent.is_empty() {
                        0
                    } else {
                        parent.matches('/').count() + 1
                    };
                    if dir_depth != parent_depth {
                        continue;
                    }

                    let name = child_dir
                        .rsplit('/')
                        .next()
                        .unwrap_or(child_dir)
                        .to_string();
                    let (file_count, additions, deletions) = dir_agg
                        .get(child_dir.as_str())
                        .copied()
                        .unwrap_or((0, 0, 0));
                    let collapsed = prev_collapsed
                        .get(child_dir.as_str())
                        .copied()
                        .unwrap_or(false);

                    result.push(TreeNode {
                        kind: TreeNodeKind::Directory {
                            name,
                            collapsed,
                            file_count,
                            total_additions: additions,
                            total_deletions: deletions,
                        },
                        depth,
                        path: child_dir.clone(),
                    });

                    if !collapsed {
                        flatten_tree(
                            child_dir,
                            depth + 1,
                            dir_children,
                            dir_files,
                            dir_agg,
                            prev_collapsed,
                            entries,
                            result,
                        );
                    }
                }
            }

            // Then, output files directly in this parent
            if !parent.is_empty() {
                if let Some(files) = dir_files.get(parent) {
                    for &entry_idx in files {
                        result.push(TreeNode {
                            kind: TreeNodeKind::File {
                                entry_index: entry_idx,
                            },
                            depth,
                            path: entries[entry_idx].path.clone(),
                        });
                    }
                }
            }
        }

        // Output root-level files first, then recurse into directories
        flatten_tree(
            "",
            0,
            &dir_children,
            &dir_files,
            &dir_agg,
            &prev_collapsed,
            entries,
            &mut self.tree_nodes,
        );

        // Append root files (files with no directory)
        for &entry_idx in &root_files {
            self.tree_nodes.push(TreeNode {
                kind: TreeNodeKind::File {
                    entry_index: entry_idx,
                },
                depth: 0,
                path: entries[entry_idx].path.clone(),
            });
        }

        // Clamp selection
        if !self.tree_nodes.is_empty() {
            self.tree_selected = self.tree_selected.min(self.tree_nodes.len() - 1);
        } else {
            self.tree_selected = 0;
        }
    }

    pub fn visible_tree_nodes(&self) -> &[TreeNode] {
        &self.tree_nodes
    }

    pub fn tree_select_up(&mut self) {
        self.tree_selected = self.tree_selected.saturating_sub(1);
    }

    pub fn tree_select_down(&mut self) {
        if !self.tree_nodes.is_empty() {
            self.tree_selected = (self.tree_selected + 1).min(self.tree_nodes.len() - 1);
        }
    }

    pub fn tree_toggle_collapse(&mut self) -> Option<usize> {
        if self.tree_nodes.is_empty() {
            return None;
        }
        let node = &self.tree_nodes[self.tree_selected];
        match &node.kind {
            TreeNodeKind::Directory { .. } => {
                let path = node.path.clone();
                // Toggle collapsed state and rebuild
                // Find the directory in tree_nodes and toggle
                for n in &mut self.tree_nodes {
                    if n.path == path {
                        if let TreeNodeKind::Directory {
                            ref mut collapsed, ..
                        } = n.kind
                        {
                            *collapsed = !*collapsed;
                            break;
                        }
                    }
                }
                self.rebuild_tree();
                // Re-select the same directory after rebuild
                for (i, n) in self.tree_nodes.iter().enumerate() {
                    if n.path == path && matches!(n.kind, TreeNodeKind::Directory { .. }) {
                        self.tree_selected = i;
                        break;
                    }
                }
                None
            }
            TreeNodeKind::File { entry_index } => Some(*entry_index),
        }
    }

    pub fn tree_collapse_all(&mut self) {
        for node in &mut self.tree_nodes {
            if let TreeNodeKind::Directory {
                ref mut collapsed, ..
            } = node.kind
            {
                *collapsed = true;
            }
        }
        self.rebuild_tree();
    }

    pub fn tree_expand_all(&mut self) {
        for node in &mut self.tree_nodes {
            if let TreeNodeKind::Directory {
                ref mut collapsed, ..
            } = node.kind
            {
                *collapsed = false;
            }
        }
        self.rebuild_tree();
    }

    pub fn selected_tree_delta_index(&self) -> Option<usize> {
        self.tree_nodes.get(self.tree_selected).and_then(|node| {
            if let TreeNodeKind::File { entry_index } = &node.kind {
                Some(*entry_index)
            } else {
                None
            }
        })
    }

    fn map_tree_selection_to_flat(&mut self) {
        if let Some(delta_idx) = self.selected_tree_delta_index() {
            if let Some(pos) = self
                .filtered_indices
                .iter()
                .position(|&i| self.entries[i].delta_index == delta_idx)
            {
                self.selected = pos;
            }
        }
    }
}

/// Simple fuzzy match: all characters of pattern must appear in text in order.
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let mut text_iter = text.chars();
    for pc in pattern.chars() {
        loop {
            match text_iter.next() {
                Some(tc) if tc == pc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

fn build_informative_path_displays(paths: &[String]) -> Vec<String> {
    let split_paths: Vec<Vec<String>> = paths
        .iter()
        .map(|p| p.split('/').map(str::to_string).collect())
        .collect();

    let mut sibling_map: HashMap<String, HashSet<String>> = HashMap::new();
    for components in &split_paths {
        if components.len() < 2 {
            continue;
        }
        for i in 0..(components.len() - 1) {
            let parent = components[..i].join("/");
            sibling_map
                .entry(parent)
                .or_default()
                .insert(components[i].clone());
        }
    }

    split_paths
        .into_iter()
        .map(|components| abbreviate_path_components(&components, &sibling_map))
        .collect()
}

fn abbreviate_path_components(
    components: &[String],
    sibling_map: &HashMap<String, HashSet<String>>,
) -> String {
    if components.len() <= 1 {
        return components.first().cloned().unwrap_or_default();
    }

    let mut out = Vec::with_capacity(components.len());
    for i in 0..(components.len() - 1) {
        let parent = components[..i].join("/");
        let siblings = sibling_map.get(&parent);
        let abbreviated = minimal_unique_prefix(&components[i], siblings);
        out.push(abbreviated);
    }

    // Keep filename fully readable; only directory components are abbreviated.
    out.push(components.last().cloned().unwrap_or_default());
    out.join("/")
}

fn minimal_unique_prefix(name: &str, siblings: Option<&HashSet<String>>) -> String {
    let Some(siblings) = siblings else {
        return first_char_or_empty(name);
    };
    if siblings.len() <= 1 {
        return first_char_or_empty(name);
    }

    let char_count = name.chars().count();
    for len in 1..=char_count {
        let prefix = take_chars(name, len);
        let unique = siblings
            .iter()
            .filter(|s| s.as_str() != name)
            .all(|s| !s.starts_with(&prefix));
        if unique {
            return prefix;
        }
    }

    name.to_string()
}

fn first_char_or_empty(s: &str) -> String {
    s.chars().next().map(|c| c.to_string()).unwrap_or_default()
}

fn take_chars(s: &str, count: usize) -> String {
    s.chars().take(count).collect()
}

fn parse_additions_deletions(display: &str) -> (usize, usize) {
    let mut additions = 0;
    let mut deletions = 0;
    for part in display.split_whitespace() {
        if let Some(num) = part.strip_prefix('+') {
            additions = num.parse().unwrap_or(0);
        } else if let Some(num) = part.strip_prefix('-') {
            deletions = num.parse().unwrap_or(0);
        }
    }
    (additions, deletions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::{FileDelta, FileStatus};
    use std::path::PathBuf;

    fn make_delta(path: &str, status: FileStatus, additions: usize, deletions: usize) -> FileDelta {
        FileDelta {
            path: PathBuf::from(path),
            old_path: None,
            status,
            hunks: Vec::new(),
            additions,
            deletions,
            binary: false,
        }
    }

    #[test]
    fn abbreviates_by_minimal_unique_prefix_under_same_parent() {
        let paths = vec![
            "src/components/navigator.rs".to_string(),
            "src/config/navigator.rs".to_string(),
        ];
        let displays = build_informative_path_displays(&paths);
        assert_eq!(displays[0], "s/com/navigator.rs");
        assert_eq!(displays[1], "s/con/navigator.rs");
    }

    #[test]
    fn uses_single_char_for_non_conflicting_directories() {
        let paths = vec!["docs/readme.md".to_string()];
        let displays = build_informative_path_displays(&paths);
        assert_eq!(displays[0], "d/readme.md");
    }

    #[test]
    fn preserves_filename_component() {
        let paths = vec![
            "src/components/navigator_super_long_name.rs".to_string(),
            "src/config/navigator_super_long_name.rs".to_string(),
        ];
        let displays = build_informative_path_displays(&paths);
        assert!(displays[0].ends_with("/navigator_super_long_name.rs"));
        assert!(displays[1].ends_with("/navigator_super_long_name.rs"));
    }

    #[test]
    fn update_from_deltas_uses_informative_path_in_display() {
        let deltas = vec![
            make_delta("src/components/navigator.rs", FileStatus::Modified, 12, 4),
            make_delta("src/config/navigator.rs", FileStatus::Added, 8, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);

        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].display, "s/com/navigator.rs [M] +12 -4");
        assert_eq!(state.entries[1].display, "s/con/navigator.rs [A] +8 -0");
        assert_eq!(state.entries[0].path, "src/components/navigator.rs");
        assert_eq!(state.entries[1].path, "src/config/navigator.rs");
    }

    #[test]
    fn toggle_tree_mode_builds_tree() {
        let deltas = vec![
            make_delta("src/components/navigator.rs", FileStatus::Modified, 12, 4),
            make_delta("src/components/diff_view.rs", FileStatus::Modified, 50, 10),
            make_delta("src/state/app_state.rs", FileStatus::Modified, 3, 1),
            make_delta("README.md", FileStatus::Modified, 5, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        assert!(!state.tree_mode);

        state.toggle_tree_mode();
        assert!(state.tree_mode);
        assert!(!state.tree_nodes.is_empty());

        // Should have directory nodes and file nodes
        let dir_count = state
            .tree_nodes
            .iter()
            .filter(|n| matches!(n.kind, TreeNodeKind::Directory { .. }))
            .count();
        let file_count = state
            .tree_nodes
            .iter()
            .filter(|n| matches!(n.kind, TreeNodeKind::File { .. }))
            .count();

        assert!(dir_count > 0);
        assert_eq!(file_count, 4);
    }

    #[test]
    fn tree_toggle_collapse_directory() {
        let deltas = vec![
            make_delta("src/components/navigator.rs", FileStatus::Modified, 12, 4),
            make_delta("src/components/diff_view.rs", FileStatus::Modified, 50, 10),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        let initial_count = state.tree_nodes.len();

        // Find first directory node
        let dir_idx = state
            .tree_nodes
            .iter()
            .position(|n| matches!(n.kind, TreeNodeKind::Directory { .. }))
            .expect("should have at least one directory");

        state.tree_selected = dir_idx;
        let result = state.tree_toggle_collapse();
        assert!(result.is_none()); // directory, not a file

        // After collapsing, there should be fewer visible nodes
        let collapsed_count = state.tree_nodes.len();
        assert!(collapsed_count < initial_count);

        // Expand again
        state.tree_toggle_collapse();
        assert_eq!(state.tree_nodes.len(), initial_count);
    }

    #[test]
    fn tree_toggle_collapse_file_returns_entry_index() {
        let deltas = vec![
            make_delta("src/main.rs", FileStatus::Modified, 5, 2),
            make_delta("README.md", FileStatus::Modified, 1, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        // Find a file node
        let file_idx = state
            .tree_nodes
            .iter()
            .position(|n| matches!(n.kind, TreeNodeKind::File { .. }))
            .expect("should have at least one file");

        state.tree_selected = file_idx;
        let result = state.tree_toggle_collapse();
        assert!(result.is_some());
    }

    #[test]
    fn tree_collapse_and_expand_all() {
        let deltas = vec![
            make_delta("src/components/navigator.rs", FileStatus::Modified, 12, 4),
            make_delta("src/state/app_state.rs", FileStatus::Modified, 3, 1),
            make_delta("docs/readme.md", FileStatus::Added, 20, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        let expanded_count = state.tree_nodes.len();

        state.tree_collapse_all();
        let collapsed_count = state.tree_nodes.len();
        assert!(collapsed_count < expanded_count);

        // All directories should be collapsed
        for node in &state.tree_nodes {
            if let TreeNodeKind::Directory { collapsed, .. } = &node.kind {
                assert!(*collapsed);
            }
        }

        state.tree_expand_all();
        assert_eq!(state.tree_nodes.len(), expanded_count);
    }

    #[test]
    fn tree_navigation() {
        let deltas = vec![
            make_delta("src/a.rs", FileStatus::Modified, 1, 0),
            make_delta("src/b.rs", FileStatus::Modified, 2, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        assert_eq!(state.tree_selected, 0);

        state.tree_select_down();
        assert_eq!(state.tree_selected, 1);

        state.tree_select_down();
        assert_eq!(state.tree_selected, 2);

        state.tree_select_up();
        assert_eq!(state.tree_selected, 1);

        // Can't go below 0
        state.tree_selected = 0;
        state.tree_select_up();
        assert_eq!(state.tree_selected, 0);
    }

    #[test]
    fn toggle_tree_mode_off_preserves_selection() {
        let deltas = vec![
            make_delta("src/a.rs", FileStatus::Modified, 1, 0),
            make_delta("src/b.rs", FileStatus::Modified, 2, 0),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);

        state.toggle_tree_mode();
        assert!(state.tree_mode);

        // Select a file node
        let file_idx = state
            .tree_nodes
            .iter()
            .position(|n| matches!(n.kind, TreeNodeKind::File { .. }))
            .unwrap();
        state.tree_selected = file_idx;

        state.toggle_tree_mode();
        assert!(!state.tree_mode);
    }

    #[test]
    fn tree_single_file_no_directory() {
        let deltas = vec![make_delta("README.md", FileStatus::Modified, 5, 0)];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        assert_eq!(state.tree_nodes.len(), 1);
        assert!(matches!(
            state.tree_nodes[0].kind,
            TreeNodeKind::File { .. }
        ));
    }

    #[test]
    fn tree_directory_aggregate_stats() {
        let deltas = vec![
            make_delta("src/a.rs", FileStatus::Modified, 10, 3),
            make_delta("src/b.rs", FileStatus::Modified, 20, 5),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        let src_dir = state
            .tree_nodes
            .iter()
            .find(|n| n.path == "src")
            .expect("should have src directory");

        if let TreeNodeKind::Directory {
            file_count,
            total_additions,
            total_deletions,
            ..
        } = &src_dir.kind
        {
            assert_eq!(*file_count, 2);
            assert_eq!(*total_additions, 30);
            assert_eq!(*total_deletions, 8);
        } else {
            panic!("expected directory node");
        }
    }

    #[test]
    fn tree_rebuild_preserves_collapsed_state() {
        let deltas = vec![
            make_delta("src/components/nav.rs", FileStatus::Modified, 10, 3),
            make_delta("src/state/app.rs", FileStatus::Modified, 5, 1),
        ];

        let mut state = NavigatorState::new();
        state.update_from_deltas(&deltas);
        state.toggle_tree_mode();

        // Collapse "src" directory
        let src_idx = state
            .tree_nodes
            .iter()
            .position(|n| n.path == "src")
            .expect("should have src dir");
        state.tree_selected = src_idx;
        state.tree_toggle_collapse();

        // Verify src is collapsed
        let src_node = state.tree_nodes.iter().find(|n| n.path == "src").unwrap();
        if let TreeNodeKind::Directory { collapsed, .. } = &src_node.kind {
            assert!(*collapsed);
        }

        // Rebuild tree (simulating a delta update)
        state.rebuild_tree();

        // src should still be collapsed
        let src_node = state.tree_nodes.iter().find(|n| n.path == "src").unwrap();
        if let TreeNodeKind::Directory { collapsed, .. } = &src_node.kind {
            assert!(*collapsed);
        }
    }
}
