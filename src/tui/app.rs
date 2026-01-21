use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Spacing,
    prelude::*,
    symbols::merge::MergeStrategy,
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    TmuxSession, create_session, rename_session, select_session,
    tui::{
        event::{AppEvent, Event, EventHandler},
        ui::components::{SessionDetails, SessionList},
    },
};

#[derive(Default)]
pub struct AppState {
    should_quit: bool,
    pub frame_count: usize,
    pub sessions: Option<Vec<TmuxSession>>,
    pub selected_session: usize,
    pub op_buffer: String,
    pub buffer_cursor: usize,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Default::default())
    }

    fn cycle_next(&mut self) {
        if let Some(sessions) = &self.sessions {
            self.selected_session = (self.selected_session + 1) % sessions.len();
        }
    }

    fn cycle_prev(&mut self) {
        if let Some(sessions) = &self.sessions {
            let new = if self.selected_session == 0 {
                sessions.len() - 1
            } else {
                self.selected_session - 1
            };

            self.selected_session = new;
        }
    }

    fn select_session(&mut self) {
        if let Some(session) = self.current_session() {
            match select_session(session) {
                Ok(_) => self.should_quit = true,
                Err(_) => {}
            }
        }
    }

    fn current_session(&self) -> Option<&TmuxSession> {
        self.sessions
            .as_ref()
            .and_then(|s| s.get(self.selected_session))
    }

    fn current_session_mut(&mut self) -> Option<&mut TmuxSession> {
        self.sessions
            .as_mut()
            .and_then(|s| s.get_mut(self.selected_session))
    }

    fn set_op_buffer(&mut self, s: &str) {
        self.op_buffer.clear();
        self.op_buffer.push_str(s);
        self.buffer_cursor = s.chars().count();
    }

    fn insert_buffer(&mut self, c: char) {
        let idx = self.byte_index();
        self.op_buffer.insert(idx, c);
        self.move_cursor_right();
    }

    fn remove_char(&mut self) {
        if self.buffer_cursor == 0 {
            return;
        }
        self.buffer_cursor -= 1;

        let byte_pos = self.byte_index();

        self.op_buffer.remove(byte_pos);
    }

    fn remove_till_start(&mut self) {
        if self.buffer_cursor == 0 {
            return;
        }
        let byte_pos = self.byte_index();
        self.buffer_cursor = 0;

        _ = self.op_buffer.drain(0..byte_pos);
    }

    fn move_cursor_left(&mut self) {
        let new_idx = self.buffer_cursor.saturating_sub(1);
        self.buffer_cursor = self.clamp_cursor(new_idx);
    }

    fn move_cursor_right(&mut self) {
        let new_idx = self.buffer_cursor.saturating_add(1);
        self.buffer_cursor = self.clamp_cursor(new_idx);
    }

    fn move_cursor_start(&mut self) {
        self.buffer_cursor = 0;
    }

    fn move_cursor_end(&mut self) {
        self.buffer_cursor = self.op_buffer.chars().count();
    }

    fn clamp_cursor(&self, pos: usize) -> usize {
        pos.clamp(0, self.op_buffer.chars().count())
    }

    fn byte_index(&self) -> usize {
        self.op_buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.buffer_cursor)
            .unwrap_or(self.op_buffer.len())
    }
}

#[derive(Debug)]
enum View {
    Normal,
    Rename,
    Create,
}

pub struct App {
    state: AppState,
    events: EventHandler,
    view: View,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: AppState::new()?,
            events: EventHandler::new(),
            view: View::Normal,
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.state.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        match self.events.next().await? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event),
                _ => {}
            },
            Event::App(event) => self.handle_app_event(event),
        }

        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match self.view {
            View::Normal => self.handle_normal_mode_key_event(event),
            View::Rename | View::Create => self.handle_input_mode_key_event(event),
        }
    }

    fn handle_input_mode_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.view = View::Normal,
            KeyCode::Char('a') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.move_cursor_start();
            }
            KeyCode::Char('e') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.move_cursor_end();
            }
            KeyCode::Char('w') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.remove_till_start();
            }
            KeyCode::Char(c) => self.state.insert_buffer(c),
            KeyCode::Backspace => self.state.remove_char(),
            KeyCode::Left => self.state.move_cursor_left(),
            KeyCode::Right => self.state.move_cursor_right(),
            KeyCode::Enter => match self.view {
                View::Rename => self.rename_session(),
                View::Create => self.create_session(),
                View::Normal => unreachable!(),
            },
            _ => {}
        }
    }

    fn handle_normal_mode_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') | KeyCode::Char('C') if event.modifiers == KeyModifiers::CONTROL => {
                self.exit()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.cycle_next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.cycle_prev();
            }
            KeyCode::Enter => {
                self.state.select_session();
            }
            KeyCode::Char('r') => {
                self.rename_mode();
            }
            KeyCode::Char('n') => {
                self.create_mode();
            }
            _ => {}
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.state.should_quit = true,
            AppEvent::TmuxSessions(sessions) => self.state.sessions = sessions,
        }
    }

    fn tick(&mut self) {
        self.state.frame_count = self.state.frame_count.wrapping_add(1);
    }

    fn rename_mode(&mut self) {
        let name = {
            let Some(session) = self.state.current_session() else {
                return;
            };
            session.name().to_string()
        };

        self.state.set_op_buffer(&name);
        self.view = View::Rename;
    }

    fn create_mode(&mut self) {
        self.state.set_op_buffer("");
        self.view = View::Create;
    }

    fn rename_session(&mut self) {
        // FIXME:
        let new_name = self.state.op_buffer.to_string();
        let Some(session) = self.state.current_session_mut() else {
            return;
        };

        if let Ok(()) = rename_session(session, &new_name) {
            self.view = View::Normal;
        }
    }

    fn create_session(&mut self) {
        // FIXME:
        let name = self.state.op_buffer.to_string();

        if let Ok(()) = create_session(&name) {
            self.view = View::Normal;
        }
    }

    fn exit(&mut self) {
        self.events.send(AppEvent::Quit);
    }

    fn draw(&self, frame: &mut Frame) {
        self.render(frame.area(), frame);
    }

    #[rustfmt::skip]
    fn keybinds(&self) -> Line<'_> {
        match self.view {
            View::Normal => {
                Line::from(vec![
                    " Up ".into(), "<K> ".blue().bold(),
                    "Down ".into(), "<J> ".blue().bold(),
                    "Switch ".into(), "<Enter> ".blue().bold(),
                    "Quit ".into(), "<Q> ".blue().bold()
                ])
            }
            View::Rename => {
                Line::from(vec![
                    " Abort ".into(), "<Esc> ".blue().bold(),
                    "Rename ".into(), "<Enter> ".blue().bold(),
                ])
            },
            View::Create => {
                Line::from(vec![
                    " Abort ".into(), "<Esc> ".blue().bold(),
                    "Create ".into(), "<Enter> ".blue().bold(),
                ])
            },
        }
    }

    fn layout(&self, area: Rect, buf: &mut Buffer) -> (Rect, Rect, Rect) {
        let area = area
            .centered_horizontally(Constraint::Length(80))
            .centered_vertically(Constraint::Length(30));

        let block = Block::default().title_bottom(self.keybinds().centered());

        block.render(area, buf);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(2), Constraint::Fill(3)])
            .spacing(Spacing::Overlap(1))
            .split(area);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1); 2])
            .spacing(Spacing::Overlap(1))
            .split(outer_layout[1]);

        let left_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_top(Line::from("Sessions").bold())
            .merge_borders(MergeStrategy::Fuzzy);
        let left_area = left_block.inner(outer_layout[0]);

        let top_right_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_top(Line::from("Details").bold().right_aligned())
            .merge_borders(MergeStrategy::Fuzzy);
        let top_right_area = top_right_block.inner(inner_layout[0]);

        let bottom_right_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .merge_borders(symbols::merge::MergeStrategy::Fuzzy);
        let bottom_right_area = bottom_right_block.inner(inner_layout[1]);

        left_block.render(outer_layout[0], buf);
        top_right_block.render(inner_layout[0], buf);
        bottom_right_block.render(inner_layout[1], buf);

        (left_area, top_right_area, bottom_right_area)
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let buf = frame.buffer_mut();
        let (left_area, top_right_area, bottom_right_area) = self.layout(area, buf);
        SessionList.render(left_area, buf, &self.state);
        SessionDetails.render(top_right_area, buf, &self.state);

        match self.view {
            View::Normal => {
                Paragraph::new(format!(
                    "{:?} {} {} {}",
                    self.view,
                    self.state.op_buffer,
                    self.state.buffer_cursor,
                    self.state.frame_count
                ))
                .render(bottom_right_area, buf);
            }
            View::Rename | View::Create => {
                let input_area = Layout::vertical([Constraint::Length(1)])
                    .horizontal_margin(1)
                    .split(bottom_right_area)[0];

                let input = Paragraph::new(self.state.op_buffer.as_str())
                    .style(Style::default().bg(Color::Gray));

                input.render(input_area, buf);
                frame.set_cursor_position(Position::new(
                    input_area.x + self.state.buffer_cursor as u16,
                    input_area.y,
                ));
            }
        }
    }
}
