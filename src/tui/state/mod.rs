mod input;
mod mode;
mod sessions;
use input::Input;
pub use mode::{Mode, ModeType};
use sessions::Sessions;

use crate::tmux;

#[derive(Default)]
pub struct State {
    should_quit: bool,
    frame_count: usize,
    repeat: u8,
    mode: Mode,
    tmux_client: tmux::Client,

    search_input: Input,
    rename_input: Input,
    create_input: Input,

    sessions: Sessions,
}
