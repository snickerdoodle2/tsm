use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub background: Color,
    pub danger: Color,
    pub dimmed_text: Color,
    pub secondary_bg: Color,
    pub secondary_text: Color,
    pub text: Color,
}

impl Default for Theme {
    fn default() -> Self {
        catppuccin::PALETTE.mocha.into()
    }
}

impl From<catppuccin::Flavor> for Theme {
    fn from(catppuccin::Flavor { colors, .. }: catppuccin::Flavor) -> Self {
        Self {
            accent: colors.green.into(),
            background: colors.crust.into(),
            danger: colors.red.into(),
            dimmed_text: colors.overlay0.into(),
            secondary_bg: colors.base.into(),
            secondary_text: colors.subtext0.into(),
            text: colors.text.into(),
        }
    }
}
