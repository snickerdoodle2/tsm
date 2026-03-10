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
        self.cursor = buf.chars().count();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn put_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.buffer.insert(idx, c);
        self.cursor_right();
    }

    pub fn remove_char(&mut self) {
        if self.cursor == 0 {
            return;
        }

        self.cursor = self.clamp(self.cursor).saturating_sub(1);

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

    pub fn cursor_start(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor = self.buffer.chars().count();
    }

    pub fn remove_till_start(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let byte_pos = self.byte_index();
        self.cursor = 0;

        let _ = self.buffer.drain(0..byte_pos);
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

#[cfg(test)]
mod tests {
    use super::Input;
    use rstest::rstest;

    #[test]
    fn set_cursor_after_the_text() {
        let mut input = Input::default();
        let text = "Lorem Ipsum";

        input.set(text);
        assert_eq!(input.buffer, text);
        assert_eq!(input.cursor, text.chars().count());
    }

    #[test]
    fn clears_buffer() {
        let mut input = Input::default();
        let text = "Lorem Ipsum";

        input.set(text);

        assert_eq!(input.buffer, text);
        input.clear();

        assert!(input.buffer.is_empty());
        assert_eq!(input.cursor, 0);
    }

    #[rstest]
    #[case(0, "aLorem Ipsum")]
    #[case(1, "Laorem Ipsum")]
    #[case(2, "Loarem Ipsum")]
    #[case(3, "Loraem Ipsum")]
    #[case(4, "Loream Ipsum")]
    #[case(5, "Lorema Ipsum")]
    #[case(6, "Lorem aIpsum")]
    #[case(7, "Lorem Iapsum")]
    #[case(8, "Lorem Ipasum")]
    #[case(9, "Lorem Ipsaum")]
    #[case(10, "Lorem Ipsuam")]
    #[case(11, "Lorem Ipsuma")]
    fn puts_char_before_cursor(#[case] cursor: usize, #[case] expected: &str) {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = cursor;

        input.put_char('a');
        assert_eq!(input.buffer, expected);
        assert_eq!(input.cursor, cursor + 1);
    }

    #[test]
    fn puts_char_oob() {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = 42;

        input.put_char('a');
        assert_eq!(input.buffer, "Lorem Ipsuma");
        assert_eq!(input.cursor, 12);
    }

    #[rstest]
    #[case(1, "orem Ipsum")]
    #[case(2, "Lrem Ipsum")]
    #[case(3, "Loem Ipsum")]
    #[case(4, "Lorm Ipsum")]
    #[case(5, "Lore Ipsum")]
    #[case(6, "LoremIpsum")]
    #[case(7, "Lorem psum")]
    #[case(8, "Lorem Isum")]
    #[case(9, "Lorem Ipum")]
    #[case(10, "Lorem Ipsm")]
    #[case(11, "Lorem Ipsu")]
    fn removes_char_before_cursor(#[case] cursor: usize, #[case] expected: &str) {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = cursor;

        input.remove_char();
        assert_eq!(input.buffer, expected);
        assert_eq!(input.cursor, cursor - 1);
    }

    #[test]
    fn removing_first_char_does_nothing() {
        let mut input = Input::default();
        let text = "Lorem Ipsum";
        input.set(text);
        input.cursor = 0;

        input.remove_char();
        assert_eq!(input.buffer, text);
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn remove_char_oob() {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = 42;

        input.remove_char();
        assert_eq!(input.buffer, "Lorem Ipsu");
        assert_eq!(input.cursor, 10);
    }

    #[test]
    fn cursor_left_move_left() {
        // duh
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor_left();

        assert_eq!(input.cursor, 10);
    }

    #[test]
    fn cursor_left_at_the_beginning_does_nothing() {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = 0;
        input.cursor_left();

        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn cursor_left_oob() {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = 42;
        input.cursor_left();

        assert_eq!(input.cursor, 11);
    }

    #[test]
    fn cursor_right_moves_right() {
        // duh
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = 0;
        input.cursor_right();

        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn cursor_right_at_the_end_does_nothing() {
        // duh
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        let cursor = input.cursor;
        input.cursor_right();

        assert_eq!(input.cursor, cursor);
    }

    #[test]
    fn cursor_right_oob() {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = 42;
        input.cursor_right();

        assert_eq!(input.cursor, 11);
    }

    #[rstest]
    fn cursor_start(#[values(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 42)] initial: usize) {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = initial;
        input.cursor_start();

        assert_eq!(input.cursor, 0);
    }

    #[rstest]
    fn cursor_end(#[values(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 42)] initial: usize) {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = initial;
        input.cursor_end();

        assert_eq!(input.cursor, 11);
    }

    #[rstest]
    #[case(0, "Lorem Ipsum")]
    #[case(1, "orem Ipsum")]
    #[case(2, "rem Ipsum")]
    #[case(3, "em Ipsum")]
    #[case(4, "m Ipsum")]
    #[case(5, " Ipsum")]
    #[case(6, "Ipsum")]
    #[case(7, "psum")]
    #[case(8, "sum")]
    #[case(9, "um")]
    #[case(10, "m")]
    #[case(11, "")]
    #[case(42, "")] // oob
    fn remove_till_start(#[case] cursor: usize, #[case] expected: &str) {
        let mut input = Input::default();
        input.set("Lorem Ipsum");
        input.cursor = cursor;
        input.remove_till_start();

        assert_eq!(input.buffer, expected);
        assert_eq!(input.cursor, 0);
    }
}
