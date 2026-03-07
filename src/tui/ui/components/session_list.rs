// FIXME: allow scrolling :(
use ratatui::{
    prelude::*,
    style::Stylize,
    text::Line,
    widgets::{List, ListItem, Widget},
};

use crate::{
    TmuxSession,
    tui::{app::PALETTE, state::AppState, ui::Spinner},
};

pub struct SessionList;

impl SessionList {
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState) {
        let Some(sessions) = state.sessions() else {
            self.render_no_list(area, buf, state);
            return;
        };

        let cur_idx = state.current_session_index().unwrap_or_default();

        let items: Vec<_> = sessions
            .enumerate()
            .map(|(i, s)| render_item(s, i, cur_idx))
            .collect();

        let list = List::new(items);

        Widget::render(list, area, buf);
    }

    fn render_no_list(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &AppState,
    ) {
        Spinner(state.frame_count()).render(area, buf);
    }
}

fn render_item(item: &TmuxSession, idx: usize, selected_idx: usize) -> ListItem<'_> {
    let relative_idx = match idx.abs_diff(selected_idx) {
        0 => Span::raw("0 "),
        x => format!(" {x}").into(),
    }
    .fg(PALETTE.overlay2);

    let name = if idx == selected_idx {
        item.name().bold().fg(PALETTE.text)
    } else {
        item.name().fg(PALETTE.subtext0)
    };

    let line = Line::from(vec![relative_idx, Span::raw(" "), name]);

    let item = ListItem::new(line);

    if idx == selected_idx {
        item.bg(PALETTE.surface0)
    } else {
        item
    }
}

impl<'a> From<&'a TmuxSession> for ListItem<'a> {
    fn from(value: &'a TmuxSession) -> Self {
        let line = Line::raw(value.name());

        ListItem::new(line)
    }
}
