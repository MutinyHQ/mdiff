use ratatui::style::Color;
use std::collections::HashMap;

const SESSION_COLORS: [Color; 8] = [
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
    Color::Green,
    Color::Blue,
    Color::Red,
    Color::LightCyan,
    Color::LightYellow,
];

pub fn session_color(index: u8) -> Color {
    SESSION_COLORS[index as usize % SESSION_COLORS.len()]
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentSession {
    pub label: String,
    pub id: String,
    pub color_index: u8,
}

#[derive(Debug, Default)]
pub struct AttributionState {
    /// Map from (file_path, hunk_index) -> AgentSession
    pub hunk_attributions: HashMap<(String, usize), AgentSession>,
    /// All known agent sessions in order
    pub sessions: Vec<AgentSession>,
    /// Whether attribution display is active
    pub active: bool,
    /// Current filter index: None = show all, Some(i) = show only session i
    pub filter_index: Option<usize>,
}

impl AttributionState {
    /// Get the session for a given file and hunk, if any.
    pub fn session_for_hunk(&self, file_path: &str, hunk_index: usize) -> Option<&AgentSession> {
        if !self.active {
            return None;
        }
        self.hunk_attributions
            .get(&(file_path.to_string(), hunk_index))
    }

    /// Cycle the filter: None -> Session 0 -> Session 1 -> ... -> None
    pub fn cycle_filter(&mut self) {
        if self.sessions.is_empty() {
            self.filter_index = None;
            return;
        }
        self.filter_index = match self.filter_index {
            None => Some(0),
            Some(i) if i + 1 < self.sessions.len() => Some(i + 1),
            Some(_) => None,
        };
    }

    /// Get the currently active filter session, if any.
    pub fn active_filter_session(&self) -> Option<&AgentSession> {
        self.filter_index.and_then(|i| self.sessions.get(i))
    }

    /// Check if a hunk passes the current filter.
    #[allow(dead_code)]
    pub fn hunk_passes_filter(&self, file_path: &str, hunk_index: usize) -> bool {
        if !self.active || self.filter_index.is_none() {
            return true;
        }
        let filter_session = match self.active_filter_session() {
            Some(s) => s,
            None => return true,
        };
        match self
            .hunk_attributions
            .get(&(file_path.to_string(), hunk_index))
        {
            Some(session) => session.id == filter_session.id,
            None => false,
        }
    }

    /// Tag a hunk with a specific session label (manual attribution).
    pub fn tag_hunk(&mut self, file_path: &str, hunk_index: usize, label: String) {
        let session_id = format!("manual:{}", label);

        let session = if let Some(existing) = self.sessions.iter().find(|s| s.id == session_id) {
            existing.clone()
        } else {
            let color_index = self.sessions.len() as u8;
            let new_session = AgentSession {
                label: label.clone(),
                id: session_id,
                color_index,
            };
            self.sessions.push(new_session.clone());
            new_session
        };

        self.hunk_attributions
            .insert((file_path.to_string(), hunk_index), session);
    }

    /// Format a compact legend string showing all sessions and their colors.
    #[allow(dead_code)]
    pub fn legend_text(&self) -> Vec<(String, Color)> {
        self.sessions
            .iter()
            .map(|s| (s.label.clone(), session_color(s.color_index)))
            .collect()
    }

    /// Get the filter status label for the context bar.
    pub fn filter_label(&self) -> String {
        match self.active_filter_session() {
            Some(s) => format!("[{}]", s.label),
            None => "[All Sessions]".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_state() -> AttributionState {
        AttributionState {
            active: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_cycle_filter_empty() {
        let mut state = active_state();
        state.cycle_filter();
        assert_eq!(state.filter_index, None);
    }

    #[test]
    fn test_cycle_filter_with_sessions() {
        let mut state = active_state();
        state.sessions.push(AgentSession {
            label: "Session #1".to_string(),
            id: "abc".to_string(),
            color_index: 0,
        });
        state.sessions.push(AgentSession {
            label: "Session #2".to_string(),
            id: "def".to_string(),
            color_index: 1,
        });

        assert_eq!(state.filter_index, None);
        state.cycle_filter();
        assert_eq!(state.filter_index, Some(0));
        state.cycle_filter();
        assert_eq!(state.filter_index, Some(1));
        state.cycle_filter();
        assert_eq!(state.filter_index, None);
    }

    #[test]
    fn test_hunk_passes_filter() {
        let mut state = active_state();
        let s1 = AgentSession {
            label: "S1".to_string(),
            id: "abc".to_string(),
            color_index: 0,
        };
        let s2 = AgentSession {
            label: "S2".to_string(),
            id: "def".to_string(),
            color_index: 1,
        };
        state.sessions.push(s1.clone());
        state.sessions.push(s2.clone());
        state
            .hunk_attributions
            .insert(("file.rs".to_string(), 0), s1);
        state
            .hunk_attributions
            .insert(("file.rs".to_string(), 1), s2);

        // No filter => all pass
        assert!(state.hunk_passes_filter("file.rs", 0));
        assert!(state.hunk_passes_filter("file.rs", 1));

        // Filter to session 0 ("abc")
        state.filter_index = Some(0);
        assert!(state.hunk_passes_filter("file.rs", 0));
        assert!(!state.hunk_passes_filter("file.rs", 1));
    }

    #[test]
    fn test_tag_hunk_creates_session() {
        let mut state = active_state();
        state.tag_hunk("file.rs", 0, "My Tag".to_string());

        assert_eq!(state.sessions.len(), 1);
        assert_eq!(state.sessions[0].label, "My Tag");
        assert!(state
            .hunk_attributions
            .contains_key(&("file.rs".to_string(), 0)));
    }

    #[test]
    fn test_session_color_cycles() {
        assert_eq!(session_color(0), Color::Cyan);
        assert_eq!(session_color(7), Color::LightYellow);
        assert_eq!(session_color(8), Color::Cyan); // wraps
    }
}
