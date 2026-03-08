use std::rc::Rc;

use anyhow::Result;
use clap::Parser;

mod args;
mod theme;

pub use theme::Theme;

#[derive(Debug, Clone)]
pub struct Config {
    pub fullscreen: bool,
    pub theme: theme::Theme,
    pub separator: Rc<str>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fullscreen: Default::default(),
            theme: Default::default(),
            separator: ";".into(),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let args = args::Args::parse();

        Ok(Self {
            fullscreen: args.fullscreen,
            ..Self::default()
        })
    }
}
