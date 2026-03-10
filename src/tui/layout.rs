use ratatui::{
    Frame,
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::{Config, tui::state::State};

pub struct Layout<'a> {
    config: &'a Config,
    state: &'a State,
}

impl<'a> Layout<'a> {
    pub fn new(config: &'a Config, state: &'a State) -> Self {
        Layout { config, state }
    }

    pub fn draw(self, frame: &mut Frame) -> Option<()> {
        let area = frame.area();
        let buf = frame.buffer_mut();

        match area.width {
            100.. => Some(()),
            40.. => Some(()),
            0.. => {
                self.small_screen(area, buf);
                None
            }
        }
    }

    fn small_screen(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Your terminal is too small")
            .centered()
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
