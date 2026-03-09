use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::{
    config::Theme,
    tui::state::{Mode, State},
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

    fn dimmed(self, set: bool) -> Self {
        if set {
            self.style(KeybindStyle::Dimmed)
        } else {
            self
        }
    }

    fn style(self, style: KeybindStyle) -> Self {
        Self { style, ..self }
    }
}

pub fn keybinds(state: &State, theme: Theme) -> Line<'_> {
    let keybinds = match state.mode() {
        Mode::Normal => {
            vec![
                Keybind::new("Up", "K"),
                Keybind::new("Down", "J"),
                Keybind::new("Create", "N"),
                Keybind::new("Rename", "R"),
                Keybind::new("Kill", "D"), //.dimmed(!state.can_delete_session()),
                Keybind::new("Switch", "Enter"),
                Keybind::new("Quit", "Q"),
            ]
        }
        Mode::Rename => {
            vec![
                Keybind::new("Abort", "Esc"),
                Keybind::new("Rename", "Enter"),
            ]
        }
        Mode::Create => {
            vec![
                Keybind::new("Abort", "Esc"),
                Keybind::new("Create", "Enter"),
            ]
        }
        Mode::Delete => {
            vec![
                Keybind::new("No", "Esc"),
                Keybind::new("Yes", "Enter").style(KeybindStyle::Danger),
            ]
        }
        Mode::Search => vec![
            Keybind::new("Cancel", "Esc"),
            Keybind::new("Search", "Enter"),
        ],
    };

    // FIXME: this suckss
    let mut line_items = Vec::with_capacity(keybinds.len() * 3);

    for Keybind { label, key, style } in keybinds {
        let (style, key_style) = match style {
            KeybindStyle::Normal => {
                let s = Style::default().bg(theme.secondary_bg);

                (s.fg(theme.text), s.fg(theme.accent))
            }
            KeybindStyle::Dimmed => {
                let s = Style::default().bg(theme.background);

                (s.fg(theme.dimmed_text), s.fg(theme.danger))
            }
            KeybindStyle::Danger => {
                let s = Style::default().bg(theme.secondary_bg);

                (s.fg(theme.text), s.fg(theme.danger))
            }
        };

        line_items.push(Span::styled(format!(" {label}"), style));
        line_items.push(Span::styled(format!(" <{key}> "), key_style));
        line_items.push(" ".into());
    }
    let _ = line_items.pop();

    Line::from(line_items)
}
