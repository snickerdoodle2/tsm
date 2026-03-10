use anyhow::Result;
use tsm::{App, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;
    let mut terminal = ratatui::init();
    let mut app = App::new(config)?;
    let res = app.run(&mut terminal).await;
    ratatui::restore();
    res
}
