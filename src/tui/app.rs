use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    symbols::border,
    widgets::Block,
};

use crate::{
    TmuxSession,
    tui::{
        event::{AppEvent, Event, EventHandler},
        ui::views::SessionList,
    },
};

#[derive(Default)]
pub struct AppState {
    should_quit: bool,
    pub frame_count: usize,
    pub sessions: Option<Vec<TmuxSession>>,
    pub selected_session: usize,
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
            KeyCode::Char('j') => {
                self.state.cycle_next();
            }
            KeyCode::Char('k') => {
                self.state.cycle_prev();
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

    fn exit(&mut self) {
        self.events.send(AppEvent::Quit);
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn title(&self) -> &'static str {
        " Sessions "
    }

    #[rustfmt::skip]
    fn keybinds(&self) -> Line<'_> {
        Line::from(vec![
            " Up ".into(), "<K> ".blue().bold(),
            "Down ".into(), "<J> ".blue().bold(),
            "Quit ".into(), "<Q> ".blue().bold()
        ])
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(self.title().bold());

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(self.keybinds().centered())
            .border_set(border::ROUNDED);

        let inner = block.inner(area);
        block.render(area, buf);

        SessionList.render(inner, buf, &self.state);
    }
}
