use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::tui::state::AppState;
pub struct SessionDetails;

impl SessionDetails {
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState) {
        let Some(session) = state.current_session() else {
            return;
        };
        Paragraph::new(format!("{:?}", session))
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
