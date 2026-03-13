use crate::{config::Theme, tmux};
use ratatui::{layout::Spacing, prelude::*, symbols::merge::MergeStrategy, widgets::Block};

pub struct LayoutPreview<'a> {
    layout: &'a tmux::Layout,
    #[allow(dead_code)]
    theme: &'a Theme,
}

impl<'a> LayoutPreview<'a> {
    pub fn new(layout: &'a tmux::Layout, theme: &'a Theme) -> Self {
        Self { layout, theme }
    }
}

impl<'a> Widget for LayoutPreview<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().merge_borders(MergeStrategy::Fuzzy);
        (&block).render(area, buf);

        let children = match self.layout.layout_type() {
            tmux::LayoutType::Horizontal(layouts) => {
                let cons = layouts
                    .iter()
                    .map(|l| Constraint::Fill(l.width()))
                    .collect::<Vec<_>>();

                Some((
                    layouts,
                    Layout::horizontal(cons).spacing(Spacing::Overlap(1)),
                ))
            }
            tmux::LayoutType::Vertical(layouts) => {
                let cons = layouts
                    .iter()
                    .map(|l| Constraint::Fill(l.height()))
                    .collect::<Vec<_>>();

                Some((layouts, Layout::vertical(cons).spacing(Spacing::Overlap(1))))
            }
            tmux::LayoutType::Pane(_) => None,
        };

        if let Some((children, layout)) = children {
            let layout = layout.split(area);
            for (c, l) in children.iter().zip(layout.iter()) {
                LayoutPreview::new(c, self.theme).render(*l, buf);
            }
        }
    }
}
