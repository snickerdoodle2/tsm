use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Whether to use full terminal size
    #[arg(long, short)]
    fullscreen: bool,
}

impl Args {
    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }
}
