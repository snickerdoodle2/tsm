use anyhow::Result;
use clap::Parser;

mod args;
mod theme;

pub use theme::Theme;

use crate::tmux;

#[derive(Debug, Default)]
pub struct Config {
    pub session_id: usize,
    pub fullscreen: bool,
    pub theme: theme::Theme,
}

impl Config {
    pub fn new() -> Result<Self> {
        let args = args::Args::parse();

        Ok(Self {
            session_id: tmux::current_session_id()?,
            fullscreen: args.fullscreen,
            ..Self::default()
        })
    }
}
