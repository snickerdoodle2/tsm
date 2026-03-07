use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::tui::{
    app::PALETTE,
    state::{AppState, View},
};

enum KeybindStyle {
    Normal,
    Dimmed,
    Danger,
}

struct Keybind {
    label: &'static str,
    key: &'static str,
    style: KeybindStyle,
}

impl Keybind {
    fn new(label: &'static str, key: &'static str) -> Self {
        Self {
            label,
            key,
            style: KeybindStyle::Normal,
        }
    }

    fn dimmed(self: Self, set: bool) -> Self {
        if set {
            self.style(KeybindStyle::Dimmed)
        } else {
            self
        }
    }

    fn style(self: Self, style: KeybindStyle) -> Self {
        Self { style, ..self }
    }
}

pub fn keybinds(state: &AppState) -> Line<'_> {
    let keybinds = match state.view() {
        View::Normal => {
            vec![
                Keybind::new("Up", "K"),
                Keybind::new("Down", "J"),
                Keybind::new("Create", "N"),
                Keybind::new("Rename", "R"),
                Keybind::new("Kill", "D").dimmed(!state.can_delete_session()),
                Keybind::new("Switch", "Enter"),
                Keybind::new("Quit", "Q"),
            ]
        }
        View::Rename => {
            vec![
                Keybind::new("Abort", "Esc"),
                Keybind::new("Rename", "Enter"),
            ]
        }
        View::Create => {
            vec![
                Keybind::new("Abort", "Esc"),
                Keybind::new("Create", "Enter"),
            ]
        }
        View::Delete => {
            vec![
                Keybind::new("No", "Esc"),
                Keybind::new("Yes", "Enter").style(KeybindStyle::Danger),
            ]
        }
        View::Search => vec![],
    };

    // FIXME: this suckss
    let mut line_items = Vec::with_capacity(keybinds.len() * 3);

    for Keybind { label, key, style } in keybinds {
        let (style, key_style) = match style {
            KeybindStyle::Normal => {
                let s = Style::default().bg(PALETTE.base.into());

                (s.fg(PALETTE.text.into()), s.fg(PALETTE.green.into()))
            }
            KeybindStyle::Dimmed => {
                let s = Style::default().bg(PALETTE.crust.into());

                (s.fg(PALETTE.surface2.into()), s.fg(PALETTE.red.into()))
            }
            KeybindStyle::Danger => {
                let s = Style::default().bg(PALETTE.base.into());

                (s.fg(PALETTE.text.into()), s.fg(PALETTE.red.into()))
            }
        };

        line_items.push(Span::styled(format!(" {label}"), style));
        line_items.push(Span::styled(format!(" <{key}> "), key_style));
        line_items.push(" ".into());
    }
    let _ = line_items.pop();

    Line::from(line_items)
}
