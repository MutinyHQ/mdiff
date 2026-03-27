/// State for the inline diff minimap.
#[derive(Debug, Clone)]
pub struct MinimapState {
    pub visible: bool,
    pub width: u16,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            visible: false,
            width: 3,
        }
    }
}

impl MinimapState {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}
