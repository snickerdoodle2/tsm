use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::{
    Config,
    tui::{
        Layout,
        event::{AppEvent, Event, EventHandler},
        state::{ModeType, State},
    },
};

pub struct App {
    config: Config,
    state: State,
    events: EventHandler,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            state: State::new(&config)?,
            config,
            events: EventHandler::new(),
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.state.should_quit() {
            let mut handle_events = false;
            terminal.draw(|frame| {
                handle_events = Layout::new(&self.config, &self.state).draw(frame).is_some();
            })?;
            self.handle_events(handle_events).await?;
        }
        Ok(())
    }

    async fn handle_events(&mut self, handle_events: bool) -> Result<()> {
        match self.events.next().await? {
            Event::Tick => self.state.tick(),
            Event::Crossterm(event) => {
                if let crossterm::event::Event::Key(key_event) = event {
                    self.handle_key_event(key_event, handle_events);
                }
            }
            Event::App(event) => self.handle_app_event(event),
        }

        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent, handle_events: bool) {
        if !handle_events {
            if event.code.is_char('q') {
                self.exit();
            }
            return;
        }

        match self.state.mode().mode_type() {
            ModeType::Normal => self.handle_normal_mode_key_event(event),
            ModeType::Confirm => self.handle_confirm_mode_key_event(event),
            ModeType::Input => self.handle_input_mode_key_event(event),
        }
    }

    fn handle_input_mode_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('a') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.cursor_start();
            }
            KeyCode::Char('e') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.cursor_end();
            }
            KeyCode::Char('w') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.remove_till_start();
            }
            KeyCode::Char(c) => self.state.put_char(c),
            KeyCode::Backspace => self.state.remove_char(),
            KeyCode::Left => self.state.cursor_left(),
            KeyCode::Right => self.state.cursor_right(),
            KeyCode::Esc => self.state.cancel_input(),
            KeyCode::Enter => self.state.submit_input(&self.events),
            _ => {}
        }
    }

    fn handle_normal_mode_key_event(&mut self, event: KeyEvent) {
        let mut digit_input = false;

        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') | KeyCode::Char('C') if event.modifiers == KeyModifiers::CONTROL => {
                self.exit();
            }

            KeyCode::Down | KeyCode::Char('j') => self.state.cycle_next(),
            KeyCode::Up | KeyCode::Char('k') => self.state.cycle_prev(),

            KeyCode::Enter => self.state.select(),
            KeyCode::Esc => self.state.cancel_search(),

            KeyCode::Char('s') => self.state.search_mode(),
            KeyCode::Char('r') => self.state.rename_mode(),
            KeyCode::Char('n') => self.state.create_mode(),
            KeyCode::Char('d') => self.state.delete_mode(),

            KeyCode::Char(digit) if digit.is_ascii_digit() => {
                digit_input = true;
                let digit = digit.to_digit(10).unwrap();
                self.state.push_repeat(digit);
            }
            _ => {}
        }

        if !digit_input {
            self.state.reset_repeat();
        }
    }

    fn handle_confirm_mode_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.state.normal_mode(),
            KeyCode::Enter => self.state.submit_confirm(&self.events),
            _ => {}
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.state.quit(),
            AppEvent::TmuxSessions => self.state.fetch(),
        }
    }

    fn exit(&mut self) {
        self.events.send(AppEvent::Quit);
    }
}
