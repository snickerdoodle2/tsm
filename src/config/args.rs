use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Whether to use full terminal size
    #[arg(long, short)]
    pub fullscreen: bool,
}
