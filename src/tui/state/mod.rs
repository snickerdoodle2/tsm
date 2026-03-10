mod input;
mod mode;
mod sessions;
use std::{mem, num::NonZeroUsize};

use anyhow::Result;
use input::Input;
pub use mode::{Mode, ModeType};
use sessions::Sessions;

use crate::{Config, tmux, tui::event::EventHandler};

#[derive(Default)]
pub struct State {
    should_quit: bool,
    frame_count: usize,
    repeat: Option<NonZeroUsize>,
    mode: Mode,
    tmux_client: tmux::Client,

    search_input: Input,
    rename_input: Input,
    create_input: Input,

    sessions: Sessions,

    session_cell: Option<tmux::Session>,
    attached_id: usize,
}

impl State {
    pub fn new(config: &Config) -> Result<Self> {
        let tmux_client = tmux::Client::new(config.separator.clone());
        Ok(Self {
            attached_id: tmux_client.current_session()?,
            tmux_client,
            ..Default::default()
        })
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
        if self.can_delete()
            && let Some(session) = self.sessions.current().cloned()
        {
            self.session_cell = Some(session);
            self.mode = Mode::Delete;
        }
    }

    // **********
    // * REPEAT *
    // **********

    pub fn push_repeat(&mut self, digit: u32) {
        self.repeat = match self.repeat {
            Some(repeat) => Some(
                repeat
                    .saturating_mul(NonZeroUsize::new(10).unwrap())
                    .saturating_add(digit as usize),
            ),
            None => NonZeroUsize::new(digit as usize),
        }
    }

    pub fn reset_repeat(&mut self) {
        self.repeat = None;
    }

    fn consume_repeat(&mut self) -> usize {
        mem::take(&mut self.repeat).map(|r| r.get()).unwrap_or(1)
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

    pub fn submit_input(&mut self, events: &EventHandler) {
        match self.mode {
            Mode::Rename => self.rename(),
            Mode::Create => self.create(events),
            Mode::Search => self.normal_mode(),
            Mode::Normal | Mode::Delete => unreachable!(),
        }
    }

    pub fn put_char(&mut self, c: char) {
        self.input_mut().put_char(c);
        self.maybe_update_filter();
    }

    pub fn remove_char(&mut self) {
        self.input_mut().remove_char();
        self.maybe_update_filter();
    }

    pub fn cursor_left(&mut self) {
        self.input_mut().cursor_left();
    }

    pub fn cursor_right(&mut self) {
        self.input_mut().cursor_right();
    }

    pub fn cursor_start(&mut self) {
        self.input_mut().cursor_start();
    }

    pub fn cursor_end(&mut self) {
        self.input_mut().cursor_end();
    }

    pub fn remove_till_start(&mut self) {
        self.input_mut().remove_till_start();
        self.maybe_update_filter();
    }

    fn input(&self) -> Option<&Input> {
        match self.mode {
            Mode::Search => Some(&self.search_input),
            Mode::Rename => Some(&self.rename_input),
            Mode::Create => Some(&self.create_input),
            Mode::Normal | Mode::Delete => None,
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

    fn maybe_update_filter(&mut self) {
        if self.mode == Mode::Search {
            self.sessions.update_filter(self.search_input.buffer());
        }
    }

    // ***********
    // * CONFIRM *
    // ***********
    pub fn submit_confirm(&mut self, events: &EventHandler) {
        match self.mode {
            Mode::Delete => self.delete(events),
            Mode::Normal | Mode::Search | Mode::Rename | Mode::Create => unreachable!(),
        }
    }

    // ************
    // * SESSIONS *
    // ************

    pub fn cycle_next(&mut self) {
        let repeat = self.consume_repeat();
        self.sessions.cycle_next(repeat);
    }

    pub fn cycle_prev(&mut self) {
        let repeat = self.consume_repeat();
        self.sessions.cycle_prev(repeat);
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

    pub fn can_delete(&self) -> bool {
        self.sessions
            .current()
            .is_some_and(|s| !s.is_attached(self.attached_id))
    }

    fn rename(&mut self) {
        if let Some(session) = mem::take(&mut self.session_cell)
            && let Some(cur_session) = self.sessions.current_mut()
            && session.id() == cur_session.id()
        {
            let _ = self
                .tmux_client
                .rename_session(cur_session, self.rename_input.buffer());
        }

        self.normal_mode();
    }

    fn create(&mut self, events: &EventHandler) {
        let name = self.create_input.buffer();
        if self.tmux_client.create_session(name).is_ok() {
            self.sessions.set_created(name);
            events.request_refetch();
        }
        self.normal_mode();
    }

    fn delete(&mut self, events: &EventHandler) {
        if let Some(session) = mem::take(&mut self.session_cell)
            && let Some(cur_session) = self.sessions.current()
            && session.id() == cur_session.id()
            && self.tmux_client.delete_session(session).is_ok()
        {
            self.sessions.set_deleted();
            events.request_refetch();
        }
        self.normal_mode();
    }

    // ********
    // * MISC *
    // ********
    pub fn debug_info(&self) -> String {
        let input = self.input();
        format!(
            r#"mode: {:?}
buffer: {:?}
cursor: {:?}
frame_count: {}
repeat: {:?}
id: {:?}"#,
            self.mode,
            input.map(Input::buffer),
            input.map(Input::cursor),
            self.frame_count,
            self.repeat,
            self.sessions.current().map(tmux::Session::id)
        )
    }
}

#[cfg(test)]
mod tests {
    // TODO: write tests
    #[test]
    fn todo() {}
}
