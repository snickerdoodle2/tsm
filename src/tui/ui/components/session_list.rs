use ratatui::{
    prelude::*,
    style::Stylize,
    text::Line,
    widgets::{List, ListItem, Widget},
};

use crate::{
    TmuxSession,
    tui::{state::AppState, ui::Spinner},
};

pub struct SessionList;

impl SessionList {
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState) {
        let Some(sessions) = state.sessions() else {
            self.render_no_list(area, buf, state);
            return;
        };

        let items: Vec<_> = sessions
            .iter()
            .enumerate()
            .map(|(i, s)| render_item(s, i, state.selected_session()))
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
    let relative_idx = idx.abs_diff(selected_idx);
    let delta = Span::styled(
        relative_idx.to_string(),
        Style::default().fg(Color::DarkGray),
    );
    let line = Line::from(vec![delta, item.name().into()]);

    let item = ListItem::new(line);

    if idx == selected_idx {
        item.bg(Color::DarkGray)
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
