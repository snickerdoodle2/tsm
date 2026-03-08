use anyhow::Result;
use tsm::{Config, tui};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;
    let mut terminal = ratatui::init();
    let mut app = tui::App::new(config)?;
    let res = app.run(&mut terminal).await;
    ratatui::restore();
    res
}
