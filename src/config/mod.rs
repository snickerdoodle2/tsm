use clap::Parser;

mod args;

#[derive(Debug, Default)]
pub struct Config {
    pub fullscreen: bool,
}

impl Config {
    pub fn new() -> Self {
        let args = args::Args::parse();

        Self {
            fullscreen: args.fullscreen,
        }
    }
}
