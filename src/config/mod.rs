use clap::Parser;

mod args;
mod theme;

pub use theme::Theme;

#[derive(Debug, Default)]
pub struct Config {
    pub fullscreen: bool,
    pub theme: theme::Theme,
}

impl Config {
    pub fn new() -> Self {
        let args = args::Args::parse();

        Self {
            fullscreen: args.fullscreen,
            ..Self::default()
        }
    }
}
