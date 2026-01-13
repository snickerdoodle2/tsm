use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{List, ListItem, Widget},
};

use crate::{
    TmuxSession,
    tui::{app::AppState, ui::Spinner},
};

pub struct SessionList;

impl SessionList {
    pub fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &AppState,
    ) {
        let Some(sessions) = &state.sessions else {
            self.render_no_list(area, buf, state);
            return;
        };

        let items: Vec<_> = sessions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let item = ListItem::from(s);
                if i == state.selected_session {
                    item.bg((255, 255, 255))
                } else {
                    item
                }
            })
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
        Spinner(state.frame_count).render(area, buf);
    }
}

impl<'a> From<&'a TmuxSession> for ListItem<'a> {
    fn from(value: &'a TmuxSession) -> Self {
        let line = Line::raw(value.name());

        ListItem::new(line)
    }
}
