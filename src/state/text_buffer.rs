#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    text: String,
    /// Byte offset into `text`, always on a char boundary.
    cursor: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    /// Create a TextBuffer pre-filled with `s`, cursor at end.
    #[allow(dead_code)]
    pub fn from(s: &str) -> Self {
        let len = s.len();
        Self {
            text: s.to_string(),
            cursor: len,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Replace entire text, cursor at end.
    pub fn set(&mut self, s: &str) {
        self.text = s.to_string();
        self.cursor = self.text.len();
    }

    /// Char-based cursor index (for rendering).
    pub fn cursor_char_index(&self) -> usize {
        self.text[..self.cursor].chars().count()
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete one character before the cursor (backspace).
    pub fn delete_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        // Find the previous char boundary
        let prev = self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.drain(prev..self.cursor);
        self.cursor = prev;
    }

    /// Delete word before cursor (Ctrl+W shell behavior):
    /// skip trailing whitespace, then delete back to next whitespace.
    pub fn delete_word_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let before: &str = &self.text[..self.cursor];
        let trimmed = before.trim_end();
        // If there was trailing whitespace, remove it first
        // Now find the start of the word (last whitespace before trimmed end)
        let word_start = trimmed
            .rfind(char::is_whitespace)
            .map(|i| {
                // Move past the whitespace char
                let c = trimmed[i..].chars().next().unwrap();
                i + c.len_utf8()
            })
            .unwrap_or(0);
        self.text.drain(word_start..self.cursor);
        self.cursor = word_start;
    }

    /// Move cursor one character left.
    pub fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.cursor = prev;
    }

    /// Move cursor one character right.
    pub fn move_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let c = self.text[self.cursor..].chars().next().unwrap();
        self.cursor += c.len_utf8();
    }

    /// Move cursor to start of text (Home / Ctrl+A).
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end of text (End / Ctrl+E).
    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let buf = TextBuffer::new();
        assert_eq!(buf.text(), "");
        assert!(buf.is_empty());
        assert_eq!(buf.cursor_char_index(), 0);
    }

    #[test]
    fn test_from() {
        let buf = TextBuffer::from("hello");
        assert_eq!(buf.text(), "hello");
        assert_eq!(buf.cursor_char_index(), 5);
    }

    #[test]
    fn test_insert_char() {
        let mut buf = TextBuffer::new();
        buf.insert_char('a');
        buf.insert_char('b');
        buf.insert_char('c');
        assert_eq!(buf.text(), "abc");
        assert_eq!(buf.cursor_char_index(), 3);
    }

    #[test]
    fn test_insert_at_middle() {
        let mut buf = TextBuffer::from("ac");
        buf.move_left(); // cursor before 'c'
        buf.insert_char('b');
        assert_eq!(buf.text(), "abc");
        assert_eq!(buf.cursor_char_index(), 2);
    }

    #[test]
    fn test_delete_back() {
        let mut buf = TextBuffer::from("abc");
        buf.delete_back();
        assert_eq!(buf.text(), "ab");
        buf.delete_back();
        assert_eq!(buf.text(), "a");
        buf.delete_back();
        assert_eq!(buf.text(), "");
        buf.delete_back(); // no-op
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn test_delete_back_middle() {
        let mut buf = TextBuffer::from("abc");
        buf.move_left(); // before 'c'
        buf.delete_back(); // delete 'b'
        assert_eq!(buf.text(), "ac");
        assert_eq!(buf.cursor_char_index(), 1);
    }

    #[test]
    fn test_move_left_right() {
        let mut buf = TextBuffer::from("abc");
        assert_eq!(buf.cursor_char_index(), 3);
        buf.move_left();
        assert_eq!(buf.cursor_char_index(), 2);
        buf.move_left();
        assert_eq!(buf.cursor_char_index(), 1);
        buf.move_right();
        assert_eq!(buf.cursor_char_index(), 2);
        buf.move_right();
        assert_eq!(buf.cursor_char_index(), 3);
        buf.move_right(); // no-op at end
        assert_eq!(buf.cursor_char_index(), 3);
    }

    #[test]
    fn test_home_end() {
        let mut buf = TextBuffer::from("hello");
        buf.move_home();
        assert_eq!(buf.cursor_char_index(), 0);
        buf.move_end();
        assert_eq!(buf.cursor_char_index(), 5);
    }

    #[test]
    fn test_clear() {
        let mut buf = TextBuffer::from("hello");
        buf.clear();
        assert_eq!(buf.text(), "");
        assert_eq!(buf.cursor_char_index(), 0);
    }

    #[test]
    fn test_set() {
        let mut buf = TextBuffer::from("old");
        buf.set("new text");
        assert_eq!(buf.text(), "new text");
        assert_eq!(buf.cursor_char_index(), 8);
    }

    #[test]
    fn test_delete_word_back_simple() {
        let mut buf = TextBuffer::from("hello world");
        buf.delete_word_back();
        assert_eq!(buf.text(), "hello ");
    }

    #[test]
    fn test_delete_word_back_trailing_spaces() {
        let mut buf = TextBuffer::from("hello   ");
        buf.delete_word_back();
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn test_delete_word_back_single_word() {
        let mut buf = TextBuffer::from("hello");
        buf.delete_word_back();
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn test_delete_word_back_multiple() {
        let mut buf = TextBuffer::from("one two three");
        buf.delete_word_back();
        assert_eq!(buf.text(), "one two ");
        buf.delete_word_back();
        assert_eq!(buf.text(), "one ");
        buf.delete_word_back();
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn test_delete_word_back_at_start() {
        let mut buf = TextBuffer::new();
        buf.delete_word_back(); // no-op
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn test_utf8_multibyte() {
        let mut buf = TextBuffer::new();
        buf.insert_char('é');
        buf.insert_char('ñ');
        assert_eq!(buf.text(), "éñ");
        assert_eq!(buf.cursor_char_index(), 2);
        buf.move_left();
        assert_eq!(buf.cursor_char_index(), 1);
        buf.delete_back();
        assert_eq!(buf.text(), "ñ");
        assert_eq!(buf.cursor_char_index(), 0);
    }

    #[test]
    fn test_utf8_emoji() {
        let mut buf = TextBuffer::new();
        buf.insert_char('🎉');
        buf.insert_char('x');
        assert_eq!(buf.text(), "🎉x");
        assert_eq!(buf.cursor_char_index(), 2);
        buf.move_left();
        buf.move_left();
        assert_eq!(buf.cursor_char_index(), 0);
        buf.move_right();
        assert_eq!(buf.cursor_char_index(), 1);
    }

    #[test]
    fn test_newline_insert() {
        let mut buf = TextBuffer::from("ab");
        buf.move_left(); // before 'b'
        buf.insert_char('\n');
        assert_eq!(buf.text(), "a\nb");
    }
}
