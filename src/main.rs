use anyhow::Result;
use tsm::tui;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = tui::App::new()?;
    let res = app.run(&mut terminal).await;
    ratatui::restore();
    res
}
