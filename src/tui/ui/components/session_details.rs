use ratatui::{
    macros::constraints,
    prelude::*,
    widgets::{Cell, Row, Table},
};

use crate::{config::Theme, tui::state::AppState};
pub struct SessionDetails;

fn row<'a>(key: &'static str, value: impl Into<Cell<'a>>, theme: Theme) -> Row<'a> {
    Row::new(vec![Cell::new(key).fg(theme.accent).bold(), value.into()])
}

impl SessionDetails {
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState, theme: Theme) {
        let Some(session) = state.current_session() else {
            return;
        };

        let widths = constraints![>=1, *=1];

        let rows = vec![
            row("Name", session.name(), theme),
            row("Created", session.created(), theme),
            row("Activity", session.last_activity(), theme),
            row("Clients", session.attached().to_string(), theme),
        ];

        let table = Table::new(rows, widths);

        Widget::render(table, area, buf);
    }
}
