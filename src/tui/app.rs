use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Spacing,
    prelude::*,
    symbols::merge::MergeStrategy,
    widgets::{Block, BorderType},
};

use crate::{
    TmuxSession,
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
    #[rustfmt::skip]
    fn keybinds(&self) -> Line<'_> {
        Line::from(vec![
            " Up ".into(), "<K> ".blue().bold(),
            "Down ".into(), "<J> ".blue().bold(),
            "Quit ".into(), "<Q> ".blue().bold()
        ])
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (left_area, top_right_area, _) = self.layout(area, buf);
        SessionList.render(left_area, buf, &self.state);
        SessionDetails.render(top_right_area, buf, &self.state);
    }
}
