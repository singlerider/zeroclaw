/// Append-only response buffer for a single message.
///
/// The agent loop pushes text (from any number of internal LLM calls).
/// The channel reads the full accumulated text via [`text()`].
/// No clears, no resets, no iteration awareness.
pub struct ResponseStream {
    buffer: String,
}

impl ResponseStream {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Append text. Leading whitespace is stripped from the first push.
    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);
        if self.buffer.len() == text.len() {
            // First push — strip leading whitespace.
            let trimmed = self.buffer.trim_start().to_string();
            self.buffer = trimmed;
        }
    }

    /// Full accumulated text so far.
    pub fn text(&self) -> &str {
        &self.buffer
    }
}
