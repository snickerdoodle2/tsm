use ratatui::{
    layout::Rect,
    macros::{constraint, constraints},
    prelude::*,
    widgets::Block,
};

use crate::config::Theme;

pub struct Modal<'a> {
    name: &'a str,
    theme: &'a Theme,
    height: u16,
    width: u16,

    area: Option<Rect>,
    keybinds: Option<Rect>,
}

impl<'a> Modal<'a> {
    pub fn new(name: &'a str, theme: &'a Theme, height: u16, width: u16) -> Self {
        Self {
            name,
            theme,
            height,
            width,
            area: None,
            keybinds: None,
        }
    }

    pub fn area(self) -> Rect {
        self.area.unwrap()
    }

    pub fn keybinds(self) -> Rect {
        self.keybinds.unwrap()
    }
}

impl<'a> Widget for &mut Modal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area
            .centered_horizontally(constraint!(<= self.width))
            .centered_vertically(constraint!(<= self.height + 1));

        let layout = Layout::vertical(constraints![==100%, ==1]).split(area);

        let block =
            Block::bordered().title_top(self.name.fg(self.theme.accent).into_centered_line());

        self.area = Some(block.inner(layout[0]));
        self.keybinds = Some(layout[1]);

        block.render(layout[0], buf);
    }
}
