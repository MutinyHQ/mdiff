/// State for the settings modal.
#[derive(Debug, Clone, Default)]
pub struct SettingsState {
    pub open: bool,
    pub selected_row: usize,
}

/// Number of setting rows in the modal.
pub const SETTINGS_ROW_COUNT: usize = 4;
