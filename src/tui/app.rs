use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    macros::constraints,
    prelude::*,
    symbols::merge::MergeStrategy,
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    Config,
    tui::{
        event::{AppEvent, Event, EventHandler},
        helpers::fill_background,
        state::{AppState, View},
        ui::components::{self, Modal, SessionDetails, SessionList},
    },
};

pub struct App {
    config: Config,
    state: AppState,
    events: EventHandler,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            state: AppState::new()?,
            events: EventHandler::new(),
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.state.should_quit() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        match self.events.next().await? {
            Event::Tick => self.state.tick(),
            Event::Crossterm(event) => {
                if let crossterm::event::Event::Key(key_event) = event {
                    self.handle_key_event(key_event);
                }
            }
            Event::App(event) => self.handle_app_event(event),
        }

        self.state.update_current_session();

        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match self.state.view() {
            View::Normal => self.handle_normal_mode_key_event(event),
            View::Delete => self.handle_confirm_mode_event(event),
            View::Search | View::Rename | View::Create => self.handle_input_mode_key_event(event),
        }
    }

    fn handle_input_mode_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('a') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.move_cursor_start();
            }
            KeyCode::Char('e') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.move_cursor_end();
            }
            KeyCode::Char('w') if event.modifiers == KeyModifiers::CONTROL => {
                self.state.remove_till_start();
            }
            KeyCode::Char(c) => self.state.insert_char(c),
            KeyCode::Backspace => self.state.remove_char(),
            KeyCode::Left => self.state.move_cursor_left(),
            KeyCode::Right => self.state.move_cursor_right(),
            KeyCode::Esc => match self.state.view() {
                View::Rename | View::Create => self.state.normal_mode(),
                View::Search => self.state.cancel_search(),
                View::Normal | View::Delete => unreachable!(),
            },
            KeyCode::Enter => match self.state.view() {
                View::Rename => self.state.rename_session(),
                View::Create => self.create_session(),
                View::Search => self.state.normal_mode(),
                View::Normal | View::Delete => unreachable!(),
            },
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

            KeyCode::Enter => self.state.select_session(),
            KeyCode::Esc => self.state.clear_search(),

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

    fn handle_confirm_mode_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.state.normal_mode(),
            KeyCode::Enter => self.delete_session(),
            _ => {}
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.state.quit(),
            AppEvent::TmuxSessions(sessions) => self.state.set_sessions(sessions),
        }
    }

    fn create_session(&mut self) {
        if self.state.create_session().is_ok() {
            self.events.request_refetch();
        }
    }

    fn delete_session(&mut self) {
        if self.state.delete_session().is_ok() {
            self.events.request_refetch();
        }
    }

    fn exit(&mut self) {
        self.events.send(AppEvent::Quit);
    }

    fn draw(&self, frame: &mut Frame) {
        self.render(frame.area(), frame);
    }

    fn get_area(&self, area: Rect) -> Rect {
        if self.config.fullscreen {
            area
        } else {
            area.centered_horizontally(Constraint::Length(100))
                .centered_vertically(Constraint::Length(30))
        }
    }

    fn layout(&self, area: Rect, buf: &mut Buffer) -> Box<[Rect]> {
        let layout = Layout::vertical(constraints![*=1, ==1]).split(area);
        components::keybinds(&self.state, self.config.theme)
            .centered()
            .render(layout[1], buf);
        let area = layout[0];

        let title_style = Style::default().bold().fg(self.config.theme.accent);

        let layout = if cfg!(feature = "debug") {
            Layout::horizontal(constraints![*=1, *=1, *=1])
        } else {
            Layout::horizontal(constraints![*=1, *=1])
        }
        .split(area);

        let left_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title_top(Line::styled("Sessions", title_style).bold())
            .merge_borders(MergeStrategy::Fuzzy);
        let left_area = left_block.inner(layout[0]);
        left_block.render(layout[0], buf);

        let middle_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title_top(Line::styled("Details", title_style).bold().left_aligned())
            .merge_borders(MergeStrategy::Fuzzy);
        let middle_area = middle_block.inner(layout[1]);
        middle_block.render(layout[1], buf);

        if cfg!(feature = "debug") {
            let right_block = Block::bordered()
                .border_type(BorderType::Plain)
                .merge_borders(symbols::merge::MergeStrategy::Fuzzy);
            let right_area = right_block.inner(layout[2]);
            right_block.render(layout[2], buf);

            Box::new([left_area, middle_area, right_area])
        } else {
            Box::new([left_area, middle_area])
        }
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let buf = frame.buffer_mut();
        let old_area = area;
        let area = self.get_area(area);
        fill_background(&old_area, &area, buf, self.config.theme);

        match self.state.view() {
            View::Normal | View::Search => {
                let splits = self.layout(area, buf);
                SessionDetails.render(splits[1], buf, &self.state, self.config.theme);

                if cfg!(feature = "debug") {
                    Paragraph::new(self.state.debug_info()).render(splits[2], buf);
                }

                let layout = Layout::vertical(constraints![*=1, ==1]).split(splits[0]);
                SessionList.render(layout[0], buf, &self.state, self.config.theme);

                let area = layout[1];
                if !self.state.search_buffer().is_empty() || self.state.view() == View::Search {
                    let input = Paragraph::new(self.state.search_buffer())
                        .bg(self.config.theme.secondary_bg);

                    input.render(area.inner(Margin::new(1, 0)), buf);
                }

                if self.state.view() == View::Search {
                    frame.set_cursor_position(Position::new(
                        area.x + self.state.search_cursor() as u16 + 1,
                        area.y,
                    ));
                }
            }
            View::Rename => {
                let title = format!(
                    "Rename {}",
                    self.state.current_session().map(|s| s.name()).unwrap_or("")
                );
                let area = Modal::new(&title)
                    .render(area, buf, &self.state, self.config.theme)
                    .centered_vertically(Constraint::Max(1));

                let input =
                    Paragraph::new(self.state.input_buffer()).bg(self.config.theme.secondary_bg);

                input.render(area.inner(Margin::new(1, 0)), buf);
                frame.set_cursor_position(Position::new(
                    area.x + self.state.input_cursor() as u16 + 1,
                    area.y,
                ));
            }
            View::Create => {
                let area = Modal::new("Create")
                    .render(area, buf, &self.state, self.config.theme)
                    .centered_vertically(Constraint::Max(1));

                let input =
                    Paragraph::new(self.state.input_buffer()).bg(self.config.theme.secondary_bg);

                input.render(area.inner(Margin::new(1, 0)), buf);
                frame.set_cursor_position(Position::new(
                    area.x + self.state.input_cursor() as u16 + 1,
                    area.y,
                ));
            }
            View::Delete => {
                let area = Modal::new("Delete")
                    .render(area, buf, &self.state, self.config.theme)
                    .centered_vertically(Constraint::Max(1));

                Paragraph::new(format!(
                    "Are you sure you want to delete {}?",
                    self.state
                        .to_delete_session()
                        .map(|s| s.name())
                        .unwrap_or("")
                ))
                .centered()
                .render(area, buf);
            }
        }
    }
}
