use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    symbols::merge::MergeStrategy,
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    Args,
    tui::{
        event::{AppEvent, Event, EventHandler},
        helpers::fill_background,
        state::{AppState, View},
        ui::components::{self, Modal, SessionDetails, SessionList},
    },
};

pub const PALETTE: catppuccin::FlavorColors = catppuccin::PALETTE.mocha.colors;

pub struct App {
    args: Args,
    state: AppState,
    events: EventHandler,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        Ok(Self {
            args,
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
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event),
                _ => {}
            },
            Event::App(event) => self.handle_app_event(event),
        }

        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match self.state.view() {
            View::Normal => self.handle_normal_mode_key_event(event),
            View::Delete => self.handle_confirm_mode_event(event),
            View::Rename | View::Create => self.handle_input_mode_key_event(event),
        }
    }

    fn handle_input_mode_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.state.normal_mode(),
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
            KeyCode::Enter => match self.state.view() {
                View::Rename => self.state.rename_session(),
                View::Create => self.create_session(),
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
                self.state.rename_mode();
            }
            KeyCode::Char('n') => {
                self.state.create_mode();
            }
            KeyCode::Char('d') => {
                self.state.delete_mode();
            }
            KeyCode::Char(digit) if digit >= '0' && digit <= '9' => {
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
            KeyCode::Enter => self.state.delete_session(),
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

    fn exit(&mut self) {
        self.events.send(AppEvent::Quit);
    }

    fn draw(&self, frame: &mut Frame) {
        self.render(frame.area(), frame);
    }

    fn get_area(&self, area: Rect) -> Rect {
        if self.args.fullscreen() {
            area
        } else {
            area.centered_horizontally(Constraint::Length(100))
                .centered_vertically(Constraint::Length(30))
        }
    }

    fn layout(&self, area: Rect, buf: &mut Buffer) -> (Rect, Rect, Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);

        components::keybinds(&self.state)
            .centered()
            .render(layout[1], buf);
        let area = layout[0];

        let title_style = Style::default().bold().fg(PALETTE.green.into());

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Max(32), Constraint::Fill(1)])
            .split(area);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1); 2])
            .split(outer_layout[1]);

        let left_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title_top(Line::styled("Sessions", title_style).bold())
            .merge_borders(MergeStrategy::Fuzzy);
        let left_area = left_block.inner(outer_layout[0]);

        let top_right_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title_top(Line::styled("Details", title_style).bold().right_aligned())
            .merge_borders(MergeStrategy::Fuzzy);
        let top_right_area = top_right_block.inner(inner_layout[0]);

        let bottom_right_block = Block::bordered()
            .border_type(BorderType::Plain)
            .merge_borders(symbols::merge::MergeStrategy::Fuzzy);
        let bottom_right_area = bottom_right_block.inner(inner_layout[1]);

        left_block.render(outer_layout[0], buf);
        top_right_block.render(inner_layout[0], buf);
        bottom_right_block.render(inner_layout[1], buf);

        (left_area, top_right_area, bottom_right_area)
    }

    fn render(&self, area: Rect, frame: &mut Frame) {
        let buf = frame.buffer_mut();
        let old_area = area.clone();
        let area = self.get_area(area);
        fill_background(&old_area, &area, buf);

        match self.state.view() {
            View::Normal => {
                let (left_area, top_right_area, bottom_right_area) = self.layout(area, buf);
                SessionList.render(left_area, buf, &self.state);
                SessionDetails.render(top_right_area, buf, &self.state);
                Paragraph::new(self.state.debug_info()).render(bottom_right_area, buf);
            }
            View::Rename => {
                let title = format!(
                    "Rename {}",
                    self.state.current_session().map(|s| s.name()).unwrap_or("")
                );
                let area = Modal::new(&title)
                    .render(area, buf, &self.state)
                    .centered_vertically(Constraint::Max(1));

                let input = Paragraph::new(self.state.buffer()).bg(PALETTE.surface0);

                input.render(area.inner(Margin::new(1, 0)), buf);
                frame.set_cursor_position(Position::new(
                    area.x + self.state.cursor() as u16 + 1,
                    area.y,
                ));
            }
            View::Create => {
                let area = Modal::new("Create")
                    .render(area, buf, &self.state)
                    .centered_vertically(Constraint::Max(1));

                let input = Paragraph::new(self.state.buffer()).bg(PALETTE.surface0);

                input.render(area.inner(Margin::new(1, 0)), buf);
                frame.set_cursor_position(Position::new(
                    area.x + self.state.cursor() as u16 + 1,
                    area.y,
                ));
            }
            View::Delete => {
                let area = Modal::new("Create")
                    .render(area, buf, &self.state)
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
