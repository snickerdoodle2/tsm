mod input;
mod mode;
mod sessions;
use input::Input;
pub use mode::{Mode, ModeType};
use sessions::Sessions;

use crate::{Config, tmux};

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

    session_cell: Option<tmux::Session>,
}

impl State {
    pub fn new(config: &Config) -> Self {
        Self {
            tmux_client: tmux::Client::new(config.separator.clone()),
            ..Default::default()
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    // ********
    // * MODE *
    // ********

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn normal_mode(&mut self) {
        debug_assert!(self.mode != Mode::Normal);
        self.mode = Mode::Normal;
    }

    pub fn search_mode(&mut self) {
        debug_assert!(self.mode == Mode::Normal);
        self.mode = Mode::Search;
    }

    pub fn rename_mode(&mut self) {
        debug_assert!(self.mode == Mode::Normal);
        if let Some(session) = self.sessions.current().cloned() {
            self.rename_input.set(session.name());
            self.session_cell = Some(session);
            self.mode = Mode::Rename;
        }
    }

    pub fn create_mode(&mut self) {
        debug_assert!(self.mode == Mode::Normal);
        self.create_input.clear();
        self.mode = Mode::Create;
    }

    pub fn delete_mode(&mut self) {
        debug_assert!(self.mode == Mode::Normal);
        if let Some(session) = self.sessions.current().cloned() {
            self.session_cell = Some(session);
            self.mode = Mode::Delete;
        }
    }

    // *********
    // * INPUT *
    // *********

    pub fn search_input(&self) -> &Input {
        &self.search_input
    }

    pub fn rename_input(&self) -> &Input {
        &self.rename_input
    }

    pub fn create_input(&self) -> &Input {
        &self.create_input
    }

    pub fn cancel_input(&mut self) {
        if self.mode == Mode::Search {
            self.cancel_search();
        }

        self.mode = Mode::Normal;
    }

    pub fn cancel_search(&mut self) {
        if !self.search_input.buffer().is_empty() {
            self.search_input.clear();
            self.sessions.update_filter("");
        }
    }

    pub fn submit_input(&mut self) {
        match self.mode {
            Mode::Rename => self.rename(),
            Mode::Create => self.create(),
            Mode::Search => self.normal_mode(),
            Mode::Normal | Mode::Delete => unreachable!(),
        }
    }

    pub fn put_char(&mut self, c: char) {
        self.input_mut().put_char(c);

        if self.mode == Mode::Search {
            self.sessions.update_filter(self.search_input.buffer());
        }
    }

    pub fn remove_char(&mut self) {
        self.input_mut().remove_char();

        if self.mode == Mode::Search {
            self.sessions.update_filter(self.search_input.buffer());
        }
    }

    pub fn cursor_left(&mut self) {
        self.input_mut().cursor_left();
    }

    pub fn cursor_right(&mut self) {
        self.input_mut().cursor_right();
    }

    fn input(&self) -> &Input {
        match self.mode {
            Mode::Search => &self.search_input,
            Mode::Rename => &self.rename_input,
            Mode::Create => &self.create_input,
            Mode::Normal | Mode::Delete => unreachable!(),
        }
    }

    fn input_mut(&mut self) -> &mut Input {
        match self.mode {
            Mode::Search => &mut self.search_input,
            Mode::Rename => &mut self.rename_input,
            Mode::Create => &mut self.create_input,
            Mode::Normal | Mode::Delete => unreachable!(),
        }
    }

    // ***********
    // * CONFIRM *
    // ***********
    pub fn submit_confirm(&mut self) {
        match self.mode {
            Mode::Delete => self.delete(),
            Mode::Normal | Mode::Search | Mode::Rename | Mode::Create => unreachable!(),
        }
    }

    // ************
    // * SESSIONS *
    // ************

    pub fn cycle_next(&mut self) {
        self.sessions.cycle_next();
    }

    pub fn cycle_prev(&mut self) {
        self.sessions.cycle_prev();
    }

    pub fn select(&mut self) {
        if let Some(session) = self.sessions.current()
            && self.tmux_client.select_session(session).is_ok()
        {
            self.should_quit = true;
        }
    }

    pub fn fetch(&mut self) {
        let sessions = self.tmux_client.list_sessions().ok();
        self.sessions.set(sessions, self.search_input.buffer());
    }

    pub fn session_cell(&self) -> Option<&tmux::Session> {
        self.session_cell.as_ref()
    }

    pub fn current(&self) -> Option<&tmux::Session> {
        self.sessions.current()
    }

    pub fn current_idx(&self) -> Option<usize> {
        self.sessions.current_idx()
    }

    pub fn sessions(&self) -> Option<impl Iterator<Item = &tmux::Session>> {
        self.sessions.sessions()
    }

    fn rename(&mut self) {
        // TODO: Implement
    }

    fn create(&mut self) {
        // TODO: Implement
    }

    fn delete(&mut self) {
        // TODO: implement
    }

    // ********
    // * MISC *
    // ********
    pub fn debug_info(&self) -> String {
        // TODO: implement
        "BOOM BOOM BOOM BOOM BOOM".to_string()
    }
}
