use serde::{Deserialize, Serialize};

use super::TextBuffer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAPair {
    pub question: String,
    pub answer: String,
    pub file_path: String,
    pub line_context: Option<String>,
}

/// Context assembled when a question is submitted.
#[derive(Debug, Clone)]
pub struct QuestionContext {
    pub file_path: String,
    pub file_language: Option<String>,
    pub visible_hunks: Vec<String>,
    pub selected_lines: Option<String>,
    pub full_diff: String,
    pub question: String,
}

#[derive(Debug, Default)]
pub struct QAState {
    pub input_open: bool,
    pub input_buffer: TextBuffer,
    pub answer_visible: bool,
    pub answer_text: String,
    pub answer_streaming: bool,
    pub history: Vec<QAPair>,
    pub history_index: Option<usize>,
    pub answer_scroll: usize,
    pub error_message: Option<String>,
}

impl QAState {
    pub fn open_input(&mut self) {
        self.input_open = true;
        self.input_buffer.clear();
    }

    pub fn close_input(&mut self) {
        self.input_open = false;
        self.input_buffer.clear();
    }

    pub fn show_answer(&mut self, question: &str, file_path: &str) {
        self.input_open = false;
        self.answer_visible = true;
        self.answer_text.clear();
        self.answer_streaming = true;
        self.answer_scroll = 0;
        self.error_message = None;
        self.history.push(QAPair {
            question: question.to_string(),
            answer: String::new(),
            file_path: file_path.to_string(),
            line_context: None,
        });
        self.history_index = Some(self.history.len() - 1);
    }

    pub fn append_answer(&mut self, text: &str) {
        self.answer_text.push_str(text);
        if let Some(idx) = self.history_index {
            if let Some(pair) = self.history.get_mut(idx) {
                pair.answer.push_str(text);
            }
        }
    }

    pub fn complete_answer(&mut self) {
        self.answer_streaming = false;
    }

    pub fn show_error(&mut self, msg: String) {
        self.input_open = false;
        self.answer_visible = true;
        self.answer_streaming = false;
        self.answer_text = msg.clone();
        self.error_message = Some(msg);
    }

    pub fn dismiss_answer(&mut self) {
        self.answer_visible = false;
        self.answer_text.clear();
        self.answer_streaming = false;
        self.answer_scroll = 0;
        self.error_message = None;
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_idx = match self.history_index {
            Some(idx) if idx > 0 => idx - 1,
            Some(0) => 0,
            None => self.history.len() - 1,
            Some(idx) => idx,
        };
        self.history_index = Some(new_idx);
        self.load_history_entry(new_idx);
    }

    pub fn history_next(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let max = self.history.len() - 1;
        let new_idx = match self.history_index {
            Some(idx) if idx < max => idx + 1,
            Some(idx) => idx,
            None => max,
        };
        self.history_index = Some(new_idx);
        self.load_history_entry(new_idx);
    }

    fn load_history_entry(&mut self, idx: usize) {
        if let Some(pair) = self.history.get(idx) {
            self.answer_text = pair.answer.clone();
            self.answer_visible = true;
            self.answer_streaming = false;
            self.answer_scroll = 0;
        }
    }

    pub fn history_display(&self) -> Option<(usize, usize)> {
        if self.history.is_empty() {
            return None;
        }
        let idx = self.history_index.unwrap_or(self.history.len() - 1);
        Some((idx + 1, self.history.len()))
    }

    pub fn current_question(&self) -> Option<&str> {
        self.history_index
            .and_then(|idx| self.history.get(idx))
            .map(|pair| pair.question.as_str())
    }
}
