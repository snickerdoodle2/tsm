use ratatui::{
    macros::constraints,
    prelude::*,
    widgets::{Cell, Row, Table, Widget},
};

use crate::{config::Theme, tmux};

pub struct SessionDetails<'a> {
    session: Option<&'a tmux::Session>,
    theme: &'a Theme,
}

impl<'a> SessionDetails<'a> {
    pub fn new(session: Option<&'a tmux::Session>, theme: &'a Theme) -> Self {
        Self { session, theme }
    }

    fn row<'b>(&self, key: &'static str, value: impl Into<Cell<'b>>) -> Row<'b> {
        Row::new(vec![
            Cell::new(key).fg(self.theme.accent).bold(),
            value.into(),
        ])
    }
}

impl<'a> Widget for SessionDetails<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(session) = self.session else {
            return;
        };

        let widths = constraints![>=1, *=1];

        let rows = vec![
            self.row("Name", session.name()),
            self.row("Created", session.created()),
            self.row("Activity", session.last_activity()),
            self.row("Clients", session.attached().to_string()),
        ];

        let table = Table::new(rows, widths);

        Widget::render(table, area, buf);
    }
}
