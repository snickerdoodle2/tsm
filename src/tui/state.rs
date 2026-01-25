use crate::TmuxSession;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum View {
    Normal,
    Rename,
    Create,
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
    selected_session: usize,

    buffer: String,
    cursor: usize,
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

    pub fn push_repeat(&mut self, digit: u32) {
        self.repeat_buffer = (self.repeat_buffer * 10) + digit;
    }

    pub fn reset_repeat(&mut self) {
        self.repeat_buffer = 0;
    }

    pub fn sessions(&self) -> Option<&Vec<TmuxSession>> {
        self.sessions.as_ref()
    }

    pub fn selected_session(&self) -> usize {
        self.selected_session
    }

    pub fn set_sessions(&mut self, sessions: Option<Vec<TmuxSession>>) {
        self.selected_session = self
            .sessions
            .as_ref()
            .and_then(|s| s.get(self.selected_session))
            .map(|s| s.id())
            .zip(sessions.as_ref())
            .and_then(|(id, sessions)| {
                sessions
                    .iter()
                    .enumerate()
                    .find_map(|(new_id, s)| (s.id() == id).then(|| new_id))
            })
            .unwrap_or(0);

        self.sessions = sessions;
    }

    pub fn rename_session(&mut self) {
        // FIXME: to_string
        let new_name = self.buffer.to_string();
        let Some(session) = self.current_session_mut() else {
            return;
        };

        if let Ok(()) = session.rename(&new_name) {
            self.view = View::Normal;
        }
    }

    pub fn create_session(&mut self) -> Result<()> {
        match TmuxSession::create(&self.buffer).context("Failed to create tmux session") {
            Ok(_) => {
                self.normal_mode();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn cycle_next(&mut self) {
        let Some(sessions) = &self.sessions.as_ref().filter(|s| s.len() > 0) else {
            return;
        };

        let to_move = self.repeat_buffer.max(1);
        self.selected_session = (sessions.len() - 1).min(self.selected_session + to_move as usize);
    }

    pub fn cycle_prev(&mut self) {
        if let None = &self.sessions.as_ref().filter(|s| s.len() > 0) {
            return;
        };

        let to_move = self.repeat_buffer.max(1);
        self.selected_session = self.selected_session.saturating_sub(to_move as usize);
    }

    pub fn select_session(&mut self) {
        if let Some(session) = self.current_session() {
            match session.select() {
                Ok(_) => self.should_quit = true,
                Err(_) => {}
            }
        }
    }

    pub fn delete_session(&mut self) {
        // FIXME: probably it's a good idea to return it with index here
        // BUG: removing last session
        let session = {
            let Some(session) = self.current_session() else {
                return;
            };

            session.clone()
        };

        if session.attached() > 0 {
            return;
        }

        if session.delete().is_ok() {
            if let Some(rest) = self.sessions.as_mut() {
                rest.retain(|s| s.name() != session.name());
            }
        }
    }

    pub fn current_session(&self) -> Option<&TmuxSession> {
        self.sessions
            .as_ref()
            .and_then(|s| s.get(self.selected_session))
    }

    pub fn current_session_mut(&mut self) -> Option<&mut TmuxSession> {
        self.sessions
            .as_mut()
            .and_then(|s| s.get_mut(self.selected_session))
    }

    pub fn set_buffer(&mut self, s: &str) {
        self.buffer.replace_range(.., s);
        self.cursor = s.chars().count();
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn insert_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.buffer.insert(idx, c);
        self.move_cursor_right();
    }

    pub fn remove_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;

        let byte_pos = self.byte_index();

        self.buffer.remove(byte_pos);
    }

    pub fn remove_till_start(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let byte_pos = self.byte_index();
        self.cursor = 0;

        _ = self.buffer.drain(0..byte_pos);
    }

    pub fn move_cursor_left(&mut self) {
        let new_idx = self.cursor.saturating_sub(1);
        self.cursor = self.clamp_cursor(new_idx);
    }

    pub fn move_cursor_right(&mut self) {
        let new_idx = self.cursor.saturating_add(1);
        self.cursor = self.clamp_cursor(new_idx);
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.buffer.chars().count();
    }

    pub fn debug_info(&self) -> String {
        format!(
            "{:?} {} {} {} {}",
            self.view, self.buffer, self.cursor, self.frame_count, self.repeat_buffer
        )
    }

    fn clamp_cursor(&self, pos: usize) -> usize {
        pos.clamp(0, self.buffer.chars().count())
    }

    fn byte_index(&self) -> usize {
        self.buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.buffer.len())
    }
}
