use ratatui::{
    Frame, layout,
    macros::{constraint, constraints},
    prelude::*,
    widgets::{Block, Paragraph, Wrap},
};

use crate::{
    Config,
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
            100.. => (true, None),
            40.. => self.medium_screen(area, buf),
            0.. => self.small_screen(area, buf),
        };

        if let Some(cursor) = cursor {
            frame.set_cursor_position(cursor);
        }

        handle_events
    }

    fn medium_screen(&self, area: Rect, buf: &mut Buffer) -> (bool, Option<(u16, u16)>) {
        let area = if !self.state.mode().is_modal() {
            let layout = layout::Layout::vertical(constraints![==100%, ==1]).split(area);
            Keybinds::new(self.state, &self.config.theme).render(layout[1], buf);
            layout[0]
        } else {
            area
        };

        let area = self.get_area(area, buf);

        let cursor = match self.state.mode() {
            Mode::Normal | Mode::Search => self.render_normal(area, buf),
            Mode::Details => self.render_details(area, buf),
            Mode::Rename => self.render_rename(area, buf),
            Mode::Create => None,
            Mode::Delete => None,
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
            Mode::Normal | Mode::Search => "Sessions",
            Mode::Details => "Details",
            Mode::Rename => "Rename",
            Mode::Create => "Create",
            Mode::Delete => "Delete",
        }
    }

    fn title_style(&self) -> Style {
        Style::default().bold().fg(self.config.theme.accent)
    }

    fn get_area(&self, area: Rect, buf: &mut Buffer) -> Rect {
        match self.state.mode() {
            Mode::Normal | Mode::Search | Mode::Details => {
                let block = Block::bordered().title_top(
                    Line::styled(self.title(self.state.mode()), self.title_style()).centered(),
                );
                (&block).render(area, buf);
                block.inner(area)
            }
            Mode::Rename | Mode::Create | Mode::Delete => {
                let mut modal =
                    Modal::new(self.title(self.state.mode()), &self.config.theme, 10, 50);
                modal.render(area, buf);
                Keybinds::new(self.state, &self.config.theme).render(modal.keybinds(), buf);
                modal.area()
            }
        }
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

    fn render_details(&self, area: Rect, buf: &mut Buffer) -> Option<(u16, u16)> {
        SessionDetails::new(self.state.current(), &self.config.theme).render(area, buf);
        None
    }
}
