use anyhow::Result;
use clap::Parser;
use tsm::{Args, tui};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut terminal = ratatui::init();
    let mut app = tui::App::new(args)?;
    let res = app.run(&mut terminal).await;
    ratatui::restore();
    res
}
