use ratatui::{layout::Flex, prelude::*};

use crate::tui::state::AppState;
pub struct SessionDetails;

impl SessionDetails {
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &AppState) {
        let Some(session) = state.current_session() else {
            return;
        };

        // FIXME: allocs

        let items: Vec<(Span<'_>, Span<'_>)> = vec![
            ("Name".into(), session.name().into()),
            ("Created".into(), session.created().to_rfc2822().into()),
            ("Attached".into(), session.attached().to_string().into()),
        ];

        let outer_layout =
            Layout::vertical(vec![Constraint::Length(1)].repeat(items.len())).split(area);

        for (i, (label, value)) in items.iter().enumerate() {
            let layout = Layout::horizontal([Constraint::Fill(1); 2])
                .flex(Flex::SpaceBetween)
                .split(outer_layout[i]);

            label.render(layout[0], buf);
            value.render(layout[1], buf);
        }
    }
}
