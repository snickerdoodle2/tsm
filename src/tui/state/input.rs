#[derive(Default)]
pub struct Input {
    buffer: String,
    cursor: usize,
}

impl Input {
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set(&mut self, buf: &str) {
        self.buffer.replace_range(.., buf);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn put_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.buffer.insert(idx, c);
    }

    pub fn remove_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;

        let idx = self.byte_index();
        self.buffer.remove(idx);
    }

    pub fn cursor_left(&mut self) {
        let idx = self.cursor.saturating_sub(1);
        self.cursor = self.clamp(idx);
    }

    pub fn cursor_right(&mut self) {
        let idx = self.cursor.saturating_add(1);
        self.cursor = self.clamp(idx);
    }

    fn byte_index(&self) -> usize {
        self.buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.buffer.len())
    }

    fn clamp(&self, idx: usize) -> usize {
        idx.clamp(0, self.buffer.chars().count())
    }
}
