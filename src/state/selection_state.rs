/// State for visual line selection in the diff view.
#[derive(Debug, Default)]
pub struct SelectionState {
    /// Whether visual mode is currently active.
    pub active: bool,
    /// The display row where selection started (anchor point).
    pub anchor: usize,
    /// The display row where the cursor currently is.
    pub cursor: usize,
}

impl SelectionState {
    /// Returns the (start, end) display row range, inclusive.
    pub fn range(&self) -> (usize, usize) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }
}
