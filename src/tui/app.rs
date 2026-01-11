use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    symbols::border,
    widgets::Block,
};

use crate::tui::{
    event::{AppEvent, Event, EventHandler},
    ui::Spinner,
};

pub struct AppState {
    should_quit: bool,
    pub frame_count: usize,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            frame_count: 0,
        })
    }
}

pub struct App {
    state: AppState,
    events: EventHandler,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: AppState::new()?,
            events: EventHandler::new(),
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
        match event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') | KeyCode::Char('C') if event.modifiers == KeyModifiers::CONTROL => {
                self.exit()
            }
            _ => {}
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.state.should_quit = true,
            AppEvent::TmuxSessions(sessions) => {}
        }
    }

    fn tick(&mut self) {
        self.state.frame_count = self.state.frame_count.wrapping_add(1);
    }

    fn exit(&mut self) {
        self.events.send(AppEvent::Quit);
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" Tmux Session Manager ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        Spinner(self.state.frame_count).render(inner, buf);
    }
}
