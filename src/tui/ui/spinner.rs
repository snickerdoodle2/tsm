use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

pub struct Spinner(pub usize);

const FRAMES: [char; 8] = ['⣾', '⢿', '⡿', '⣷', '⣯', '⢟', '⡻', '⣽'];

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame = (self.0 & 0b1111) >> 1;
        let mut char_buf = [0; 3];
        let frame = FRAMES[frame].encode_utf8(&mut char_buf);
        Paragraph::new(frame as &_).render(area, buf);
    }
}
