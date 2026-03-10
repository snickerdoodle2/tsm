use ratatui::{
    prelude::*,
    widgets::{List, ListItem, Widget},
};

use crate::{config::Theme, tmux, tui::state::State};

pub struct SessionList<'a> {
    state: &'a State,
    theme: &'a Theme,
}

impl<'a> SessionList<'a> {
    pub fn new(state: &'a State, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    fn render_session(
        &self,
        session: &'a tmux::Session,
        idx: usize,
        selected_idx: usize,
    ) -> ListItem<'a> {
        let relative_idx = match idx.abs_diff(selected_idx) {
            0 => Span::raw("0 "),
            x => format!(" {x}").into(),
        }
        .fg(self.theme.secondary_text);

        let name = if idx == selected_idx {
            session.name().bold().fg(self.theme.text)
        } else {
            session.name().fg(self.theme.secondary_text)
        };

        let line = Line::from(vec![relative_idx, Span::raw(" "), name]);

        let session = ListItem::new(line);

        if idx == selected_idx {
            session.bg(self.theme.secondary_bg)
        } else {
            session
        }
    }
}

impl<'a> Widget for SessionList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(sessions) = self.state.sessions() else {
            return;
        };

        let cur_idx = self.state.current_idx().unwrap_or_default();

        let items: Vec<_> = sessions
            .enumerate()
            .map(|(i, s)| self.render_session(s, i, cur_idx))
            .collect();

        Widget::render(List::new(items), area, buf);
    }
}
