use super::TextBuffer;

#[derive(Debug, Clone)]
pub struct CommandBarState {
    pub active: bool,
    pub buffer: TextBuffer,
}

impl CommandBarState {
    pub fn new() -> Self {
        Self {
            active: false,
            buffer: TextBuffer::new(),
        }
    }

    pub fn open(&mut self) {
        self.active = true;
        self.buffer.clear();
    }

    pub fn close(&mut self) {
        self.active = false;
        self.buffer.clear();
    }
}

impl Default for CommandBarState {
    fn default() -> Self {
        Self::new()
    }
}
