// FIXME: probably should update session list only after keystroke
use crate::TmuxSession;
use anyhow::{Context, Result, bail};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::mem;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum View {
    Normal,
    Rename,
    Create,
    Delete,
    Search,
}

impl Default for View {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Default)]
pub struct AppState {
    should_quit: bool,
    frame_count: usize,
    view: View,
    repeat_buffer: u32,

    sessions: Option<Vec<TmuxSession>>,
    // TODO: add ouroboros (or similar crate) to fix searching...
    /// Selected session's id
    selected_session: Option<usize>,
    to_delete_session: Option<TmuxSession>,

    input_buffer: String,
    input_cursor: usize,

    search_buffer: String,
    search_cursor: usize,

    matcher: SkimMatcherV2,
}

impl AppState {
    pub fn new() -> Result<Self> {
        Ok(Default::default())
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn normal_mode(&mut self) {
        debug_assert!(self.view != View::Normal);
        self.view = View::Normal;
    }

    pub fn search_mode(&mut self) {
        debug_assert!(self.view == View::Normal);
        self.view = View::Search;
    }

    pub fn cancel_search(&mut self) {
        self.clear_search();
        self.view = View::Normal;
    }

    pub fn rename_mode(&mut self) {
        debug_assert!(self.view == View::Normal);
        let Some(name) = self.current_session().map(|s| s.name()) else {
            return;
        };

        // NOTE: inlining both functions would allow omitting allocating here
        // buuut i kinda don't want to (+ i'll probably use Rc<str> anyway)
        self.set_buffer(&name.to_string());
        self.view = View::Rename;
    }

    pub fn create_mode(&mut self) {
        debug_assert!(self.view == View::Normal);
        self.set_buffer("");
        self.view = View::Create;
    }

    pub fn delete_mode(&mut self) {
        self.view = View::Delete;
        self.to_delete_session = self.current_session().cloned();
    }

    pub fn push_repeat(&mut self, digit: u32) {
        self.repeat_buffer = (self.repeat_buffer * 10) + digit;
    }

    pub fn reset_repeat(&mut self) {
        self.repeat_buffer = 0;
    }

    pub fn sessions(&self) -> Option<impl Iterator<Item = &TmuxSession>> {
        self.sessions.as_ref().map(|s| {
            s.iter().filter(|s| {
                self.matcher
                    .fuzzy_match(s.name(), &self.search_buffer)
                    .is_some()
            })
        })
    }

    fn sessions_mut(&mut self) -> Option<impl Iterator<Item = &mut TmuxSession>> {
        self.sessions.as_mut().map(|s| {
            s.iter_mut()
                .filter(|s| s.name().starts_with(&self.search_buffer))
        })
    }

    pub fn set_sessions(&mut self, sessions: Option<Vec<TmuxSession>>) {
        self.sessions = sessions;
        self.update_current_session();
    }

    pub fn update_current_session(&mut self) {
        if self.current_session().is_none() {
            self.selected_session = self.sessions().and_then(|mut s| s.next().map(|s| s.id()));
        }
    }

    pub fn rename_session(&mut self) {
        // FIXME: to_string
        let new_name = self.input_buffer.to_string();
        let Some(session) = self.current_session_mut() else {
            return;
        };

        if let Ok(()) = session.rename(&new_name) {
            self.view = View::Normal;
        }
    }

    pub fn create_session(&mut self) -> Result<()> {
        match TmuxSession::create(&self.input_buffer).context("Failed to create tmux session") {
            Ok(_) => {
                self.normal_mode();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn cycle_next(&mut self) {
        self.selected_session = if let Some(idx) = self.current_session_index() {
            let to_move = self.repeat_buffer.max(1);
            let sessions: Vec<_> = self.sessions().map(|s| s.collect()).unwrap_or_default();
            let new_idx = (sessions.len() - 1).min(idx + to_move as usize);

            Some(sessions[new_idx].id())
        } else {
            self.sessions().and_then(|mut s| s.next().map(|s| s.id()))
        };
    }

    pub fn cycle_prev(&mut self) {
        self.selected_session = if let Some(idx) = self.current_session_index() {
            let to_move = self.repeat_buffer.max(1);
            let sessions: Vec<_> = self.sessions().map(|s| s.collect()).unwrap_or_default();
            let new_idx = idx.saturating_sub(to_move as usize);

            Some(sessions[new_idx].id())
        } else {
            self.sessions().and_then(|s| s.last().map(|s| s.id()))
        };
    }

    pub fn select_session(&mut self) {
        if let Some(session) = self.current_session() {
            match session.select() {
                Ok(_) => self.should_quit = true,
                Err(_) => {}
            }
        }
    }

    pub fn delete_session(&mut self) -> Result<()> {
        let Some(session) = mem::take(&mut self.to_delete_session) else {
            bail!("No session to delete");
        };

        if session.attached() > 0 {
            bail!("Someone is attached to the session");
        }

        session.delete()?;

        self.normal_mode();
        Ok(())
    }

    pub fn can_delete_session(&self) -> bool {
        self.current_session().is_some_and(|s| s.attached() == 0)
    }

    pub fn current_session(&self) -> Option<&TmuxSession> {
        let selected_id = self.selected_session?;
        let mut sessions = self.sessions()?;

        sessions.find(|s| s.id() == selected_id)
    }

    pub fn current_session_mut(&mut self) -> Option<&mut TmuxSession> {
        let selected_id = self.selected_session?;
        let mut sessions = self.sessions_mut()?;

        sessions.find(|s| s.id() == selected_id)
    }

    pub fn current_session_index(&self) -> Option<usize> {
        let selected_id = self.selected_session?;
        let sessions = self.sessions()?;

        sessions
            .enumerate()
            .find(|(_, s)| s.id() == selected_id)
            .map(|(i, _)| i)
    }

    pub fn to_delete_session(&self) -> Option<&TmuxSession> {
        self.to_delete_session.as_ref()
    }

    pub fn search_buffer(&self) -> &str {
        &self.search_buffer
    }

    pub fn clear_search(&mut self) {
        self.search_buffer.clear();
        self.search_cursor = 0;
    }

    pub fn search_cursor(&self) -> usize {
        self.search_cursor
    }

    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn input_cursor(&self) -> usize {
        self.input_cursor
    }

    fn set_buffer(&mut self, s: &str) {
        self.buffer_mut().replace_range(.., s);
        *self.cursor_mut() = s.chars().count();
    }

    fn buffer(&self) -> &str {
        if self.view == View::Search {
            &self.search_buffer
        } else {
            &self.input_buffer
        }
    }

    fn buffer_mut(&mut self) -> &mut String {
        if self.view == View::Search {
            &mut self.search_buffer
        } else {
            &mut self.input_buffer
        }
    }

    fn cursor(&self) -> usize {
        if self.view == View::Search {
            self.search_cursor
        } else {
            self.input_cursor
        }
    }

    fn cursor_mut(&mut self) -> &mut usize {
        if self.view == View::Search {
            &mut self.search_cursor
        } else {
            &mut self.input_cursor
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.buffer_mut().insert(idx, c);
        self.move_cursor_right();
    }

    pub fn remove_char(&mut self) {
        if self.cursor() == 0 {
            return;
        }
        *self.cursor_mut() -= 1;

        let byte_pos = self.byte_index();

        self.buffer_mut().remove(byte_pos);
    }

    pub fn remove_till_start(&mut self) {
        if self.cursor() == 0 {
            return;
        }
        let byte_pos = self.byte_index();
        *self.cursor_mut() = 0;

        _ = self.buffer_mut().drain(0..byte_pos);
    }

    pub fn move_cursor_left(&mut self) {
        let new_idx = self.cursor().saturating_sub(1);
        *self.cursor_mut() = self.clamp_cursor(new_idx);
    }

    pub fn move_cursor_right(&mut self) {
        let new_idx = self.cursor().saturating_add(1);
        *self.cursor_mut() = self.clamp_cursor(new_idx);
    }

    pub fn move_cursor_start(&mut self) {
        *self.cursor_mut() = 0
    }

    pub fn move_cursor_end(&mut self) {
        *self.cursor_mut() = self.buffer().chars().count();
    }

    pub fn debug_info(&self) -> String {
        format!(
            "view: {:?}\nbuffer: {}\ncursor: {}\nframe_count: {}\nrepeat: {}\nid: {}\nname: {}",
            self.view,
            self.buffer(),
            self.cursor(),
            self.frame_count,
            self.repeat_buffer,
            self.current_session().map(|s| s.id()).unwrap_or_default(),
            self.current_session().map(|s| s.name()).unwrap_or_default(),
        )
    }

    fn clamp_cursor(&self, pos: usize) -> usize {
        pos.clamp(0, self.buffer().chars().count())
    }

    fn byte_index(&self) -> usize {
        self.buffer()
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor())
            .unwrap_or(self.buffer().len())
    }
}
