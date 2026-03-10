use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

use crate::{config::Theme, tui::state};

pub struct Input<'a> {
    input: &'a state::Input,
    theme: &'a Theme,
    x: u16,
    y: u16,
}

impl<'a> Input<'a> {
    pub fn new(input: &'a state::Input, theme: &'a Theme) -> Self {
        Self {
            input,
            theme,
            x: 0,
            y: 0,
        }
    }

    pub fn cursor(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

impl<'a> Widget for &mut Input<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input = Paragraph::new(self.input.buffer()).bg(self.theme.secondary_bg);
        self.x = area.x + self.input.cursor() as u16;
        self.y = area.y;
        input.render(area, buf);
    }
}
