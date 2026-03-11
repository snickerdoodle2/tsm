use ratatui::{layout::Rect, macros::constraint, prelude::*};

use crate::{
    config::Theme,
    tui::state::{self, Mode},
};

enum KeybindStyle {
    Normal,
    Dimmed,
    Danger,
}

struct Keybind<'a> {
    label: &'static str,
    key: &'static str,
    style: KeybindStyle,
    theme: &'a Theme,
}

impl<'a> Keybind<'a> {
    fn new(label: &'static str, key: &'static str, theme: &'a Theme) -> Self {
        Self {
            label,
            key,
            style: KeybindStyle::Normal,
            theme,
        }
    }

    fn style(self, style: KeybindStyle, set: bool) -> Self {
        if set { Self { style, ..self } } else { self }
    }

    fn width(&self) -> u16 {
        self.label.chars().count() as u16 + self.key.chars().count() as u16 + 2 + 3
    }
}

impl<'a> Widget for Keybind<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (style, kb_style, bg_style) = match self.style {
            KeybindStyle::Normal => (
                Style::default().fg(self.theme.text),
                Style::default().fg(self.theme.accent),
                Style::default().bg(self.theme.secondary_bg),
            ),
            KeybindStyle::Dimmed => (
                Style::default().fg(self.theme.dimmed_text),
                Style::default().fg(self.theme.danger),
                Style::default().bg(self.theme.background),
            ),
            KeybindStyle::Danger => (
                Style::default().fg(self.theme.text),
                Style::default().fg(self.theme.danger),
                Style::default().bg(self.theme.secondary_bg),
            ),
        };

        Line::from(vec![
            Span::raw(" "),
            Span::styled(self.label, style),
            Span::raw(" "),
            Span::styled(["<", self.key, ">"].join(""), kb_style),
            Span::raw(" "),
        ])
        .style(bg_style)
        .render(area, buf);
    }
}

pub struct Keybinds<'a> {
    state: &'a state::State,
    theme: &'a Theme,
}

impl<'a> Keybinds<'a> {
    pub fn new(state: &'a state::State, theme: &'a Theme) -> Self {
        Self { state, theme }
    }
}

impl<'a> Widget for Keybinds<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let keybinds = match self.state.mode() {
            Mode::Normal => {
                vec![
                    Keybind::new("Up", "K", self.theme),
                    Keybind::new("Down", "J", self.theme),
                    Keybind::new("Create", "N", self.theme),
                    Keybind::new("Rename", "R", self.theme),
                    Keybind::new("Kill", "D", self.theme)
                        .style(KeybindStyle::Dimmed, !self.state.can_delete()),
                    Keybind::new("Switch", "Enter", self.theme),
                    Keybind::new("Quit", "Q", self.theme),
                ]
            }
            Mode::Details => {
                vec![Keybind::new("Quit", "Q", self.theme)]
            }
            Mode::Rename => {
                vec![
                    Keybind::new("Abort", "Esc", self.theme),
                    Keybind::new("Rename", "Enter", self.theme),
                ]
            }
            Mode::Create => {
                vec![
                    Keybind::new("Abort", "Esc", self.theme),
                    Keybind::new("Create", "Enter", self.theme),
                ]
            }
            Mode::Delete => {
                vec![
                    Keybind::new("No", "Esc", self.theme),
                    Keybind::new("Yes", "Enter", self.theme).style(KeybindStyle::Danger, true),
                ]
            }
            Mode::Search => vec![
                Keybind::new("Cancel", "Esc", self.theme),
                Keybind::new("Search", "Enter", self.theme),
            ],
        };

        let mut constraints = Vec::with_capacity(keybinds.len());
        let max_width = area.width;
        let mut width = 0;

        for kb in &keybinds {
            let kb_width = kb.width();
            // Do not show if it would overflow width
            if width + kb_width + 1 >= max_width {
                break;
            }
            width += kb_width + 1;
            constraints.push(constraint!(==kb_width));
        }
        width -= 1;

        let layout = Layout::horizontal(constraints)
            .spacing(1)
            .split(area.centered_horizontally(constraint!(==width)));

        for (k, l) in keybinds.into_iter().zip(layout.iter()) {
            k.render(*l, buf);
        }
    }
}
