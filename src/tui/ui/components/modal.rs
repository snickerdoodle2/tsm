use ratatui::{buffer::Buffer, layout::Rect, macros::constraints, prelude::*, widgets::Block};

use crate::{
    config::Theme,
    tui::{helpers::fill_background, state::AppState, ui::components::keybinds},
};

pub struct Modal<'a>(&'a str);

impl<'a> Modal<'a> {
    pub fn new(name: &'a str) -> Self {
        Self(name)
    }

    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState, theme: Theme) -> Rect {
        let old_area = area;
        let area = area
            .centered_horizontally(Constraint::Ratio(1, 2))
            .centered_vertically(Constraint::Ratio(1, 2));
        fill_background(&old_area, &area, buf, theme);

        let layout = Layout::vertical(constraints![==100%, ==1]).split(area);
        keybinds(state, theme).centered().render(layout[1], buf);

        let block = Block::bordered().title_top(self.0.fg(theme.accent));
        let rect = block.inner(layout[0]);
        block.render(layout[0], buf);

        rect
    }
}
