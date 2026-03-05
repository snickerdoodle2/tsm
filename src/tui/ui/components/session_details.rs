use ratatui::{
    macros::constraints,
    prelude::*,
    widgets::{Cell, Row, Table},
};

use crate::tui::{app::PALETTE, state::AppState};
pub struct SessionDetails;

fn row<'a>(key: &'static str, value: impl Into<Cell<'a>>) -> Row<'a> {
    Row::new(vec![Cell::new(key).fg(PALETTE.green).bold(), value.into()])
}

impl SessionDetails {
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState) {
        let Some(session) = state.current_session() else {
            return;
        };

        let widths = constraints![>=1, *=1];

        let rows = vec![
            row("Name", session.name()),
            row("Created", session.created()),
            row("Activity", session.last_activity()),
            row("Clients", session.attached().to_string()),
        ];

        let table = Table::new(rows, widths);

        Widget::render(table, area, buf);
    }
}
