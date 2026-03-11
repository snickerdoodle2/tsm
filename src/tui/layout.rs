use std::rc::Rc;

use ratatui::{
    Frame, layout,
    macros::{constraint, constraints},
    prelude::*,
    symbols::braille::BRAILLE,
    widgets::{Block, Paragraph, Wrap},
};

use crate::{
    Config,
    config::Theme,
    tui::{
        components::{Input, Keybinds, Modal, SessionDetails, SessionList},
        state::{Mode, State},
    },
};

pub struct Layout<'a> {
    config: &'a Config,
    state: &'a State,
}

impl<'a> Layout<'a> {
    pub fn new(config: &'a Config, state: &'a State) -> Self {
        Layout { config, state }
    }

    pub fn draw(self, frame: &mut Frame) -> bool {
        let area = frame.area();
        let buf = frame.buffer_mut();

        let (handle_events, cursor) = match area.width {
            100.. => self.large_screen(area, buf),
            40.. => self.medium_screen(area, buf),
            0.. => self.small_screen(area, buf),
        };

        if let Some(cursor) = cursor {
            frame.set_cursor_position(cursor);
        }

        handle_events
    }

    fn large_screen(&self, area: Rect, buf: &mut Buffer) -> (bool, Option<(u16, u16)>) {
        if self.state.mode().is_modal() {
            let area = self.get_area(self.state.mode(), area, buf);
            let cursor = match self.state.mode() {
                Mode::Rename => self.render_rename(area, buf),
                Mode::Create => self.render_create(area, buf),
                Mode::Delete => self.render_delete(area, buf),
                Mode::Normal | Mode::Details | Mode::Search => unreachable!(),
                #[cfg(feature = "debug")]
                Mode::Debug => unreachable!(),
            };

            return (true, cursor);
        }

        let layout = layout::Layout::vertical(constraints![*=1, ==1]).split(area);
        Keybinds::new(self.state, &self.config.theme).render(layout[1], buf);
        let layout = self.large_layout(layout[0]);

        let mut cursor = None;

        let block = self.get_area(Mode::Normal, layout[0], buf);
        if let Some(c) = self.render_normal(block, buf) {
            cursor = Some(c);
        }

        let block = self.get_area(Mode::Details, layout[1], buf);
        if let Some(c) = self.render_details(block, buf) {
            cursor = Some(c);
        }

        #[cfg(feature = "debug")]
        {
            let block = self.get_area(Mode::Debug, layout[2], buf);
            if let Some(c) = self.render_debug(block, buf) {
                cursor = Some(c);
            }
        }

        (true, cursor)
    }

    fn large_layout(&self, area: Rect) -> Rc<[Rect]> {
        let layout = if cfg!(feature = "debug") {
            layout::Layout::horizontal(constraints![*=1, *=1, *=1])
        } else {
            layout::Layout::horizontal(constraints![*=1, *=1])
        };

        layout.split(area)
    }

    fn medium_screen(&self, area: Rect, buf: &mut Buffer) -> (bool, Option<(u16, u16)>) {
        let area = if !self.state.mode().is_modal() {
            let layout = layout::Layout::vertical(constraints![==100%, ==1]).split(area);
            Keybinds::new(self.state, &self.config.theme).render(layout[1], buf);
            layout[0]
        } else {
            area
        };

        let area = self.get_area(self.state.mode(), area, buf);

        let cursor = match self.state.mode() {
            Mode::Normal | Mode::Search => self.render_normal(area, buf),
            Mode::Details => self.render_details(area, buf),
            Mode::Rename => self.render_rename(area, buf),
            Mode::Create => self.render_create(area, buf),
            Mode::Delete => self.render_delete(area, buf),
            #[cfg(feature = "debug")]
            Mode::Debug => self.render_debug(area, buf),
        };

        (true, cursor)
    }

    fn small_screen(&self, area: Rect, buf: &mut Buffer) -> (bool, Option<(u16, u16)>) {
        Paragraph::new("Your terminal is too small")
            .centered()
            .wrap(Wrap { trim: true })
            .render(area, buf);

        (false, None)
    }

    fn title(&self, mode: Mode) -> &'static str {
        match mode {
            Mode::Normal | Mode::Search => " Sessions ",
            Mode::Details => " Details ",
            Mode::Rename => " Rename ",
            Mode::Create => " Create ",
            Mode::Delete => " Delete ",
            #[cfg(feature = "debug")]
            Mode::Debug => " Debug ",
        }
    }

    fn title_style(&self) -> Style {
        Style::default().bold().fg(self.config.theme.accent)
    }

    fn get_area(&self, mode: Mode, area: Rect, buf: &mut Buffer) -> Rect {
        match mode {
            Mode::Normal | Mode::Search | Mode::Details => self.full_area(mode, area, buf),
            Mode::Rename | Mode::Create | Mode::Delete => self.modal_area(mode, area, buf),
            #[cfg(feature = "debug")]
            Mode::Debug => self.full_area(mode, area, buf),
        }
    }

    fn full_area(&self, mode: Mode, area: Rect, buf: &mut Buffer) -> Rect {
        let border_style = if mode == self.state.mode() {
            Style::default().fg(self.config.theme.accent)
        } else {
            Style::default()
        };

        let block = Block::bordered()
            .border_style(border_style)
            .title_top(Line::styled(self.title(mode), self.title_style()).centered());
        (&block).render(area, buf);
        block.inner(area)
    }

    fn modal_area(&self, mode: Mode, area: Rect, buf: &mut Buffer) -> Rect {
        let mut modal = Modal::new(self.title(mode), &self.config.theme, 10, 50);
        modal.render(area, buf);
        let new_area = modal.area();
        fill_background(area, new_area, buf, &self.config.theme);
        Keybinds::new(self.state, &self.config.theme).render(modal.keybinds(), buf);
        new_area
    }

    fn render_normal(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        let layout = layout::Layout::vertical(constraints![*=1, ==1]).split(area);

        SessionList::new(self.state, &self.config.theme).render(layout[0], buf);

        let mut cursor = None;

        if self.state.mode() == Mode::Search || !self.state.search_input().buffer().is_empty() {
            let mut input = Input::new(self.state.search_input(), &self.config.theme);
            input.render(layout[1].inner(Margin::new(1, 0)), buf);
            if self.state.mode() == Mode::Search {
                cursor = Some(input.cursor());
            }
        }

        cursor
    }

    fn render_rename(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        let layout = layout::Layout::vertical(constraints![== 1, == 1])
            .spacing(1)
            .split(area.centered_vertically(constraint![== 3]));

        Paragraph::new(format!(
            r#"Rename "{}""#,
            self.state.current().map(|s| s.name()).unwrap_or_default()
        ))
        .wrap(Wrap { trim: true })
        .centered()
        .render(layout[0], buf);

        let mut input = Input::new(self.state.rename_input(), &self.config.theme);
        input.render(layout[1].inner(Margin::new(1, 0)), buf);

        Some(input.cursor())
    }

    fn render_create(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        let layout = layout::Layout::vertical(constraints![== 1, == 1])
            .spacing(1)
            .split(area.centered_vertically(constraint![== 3]));

        Paragraph::new("New session name")
            .wrap(Wrap { trim: true })
            .centered()
            .render(layout[0], buf);

        let mut input = Input::new(self.state.create_input(), &self.config.theme);
        input.render(layout[1].inner(Margin::new(1, 0)), buf);

        Some(input.cursor())
    }

    fn render_delete(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        let area = area.centered_vertically(constraint![== 1]);

        Paragraph::new(format!(
            r#"Are you sure you want to delete "{}"?"#,
            self.state.current().map(|s| s.name()).unwrap_or_default()
        ))
        .wrap(Wrap { trim: true })
        .centered()
        .render(area, buf);

        None
    }

    fn render_details(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        SessionDetails::new(self.state.current(), &self.config.theme).render(area, buf);
        None
    }

    #[cfg(feature = "debug")]
    fn render_debug(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        Paragraph::new(self.state.debug_info()).render(area, buf);
        None
    }
}

fn fill_background(old: Rect, new: Rect, buf: &mut Buffer, theme: &Theme) {
    let new = new.outer(Margin::new(1, 1));
    let style = Style::default().fg(theme.secondary_bg);
    for y in old.top()..old.bottom() {
        for x in old.left()..old.right() {
            if !(x >= new.left() && x < new.right() && y >= new.top() && y < new.bottom()) {
                buf[(x, y)]
                    .set_char(BRAILLE[bg_noise(x, y) % BRAILLE.len()])
                    .set_style(style);
            }
        }
    }
}

// NOTE: no idea what's happening here
// it was generated by llm
fn bg_noise(a: u16, b: u16) -> usize {
    let mut x = ((a as u32) << 16) | (b as u32);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b3c6d);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297a2d39);
    x ^= x >> 15;

    x as usize
}
