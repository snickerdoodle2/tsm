mod mode;
pub use mode::{Mode, ModeType};

use crate::tmux;

#[derive(Default)]
pub struct State {
    should_quit: bool,
    frame_count: usize,
    repeat: u8,
    mode: Mode,
    tmux_client: tmux::Client,
}
