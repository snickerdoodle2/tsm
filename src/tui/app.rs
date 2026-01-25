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
    Args,
    tui::{
        event::{AppEvent, Event, EventHandler},
        state::{AppState, View},
        ui::components::{SessionDetails, SessionList},
    },
};

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
                View::Normal => unreachable!(),
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
                // TODO: confirmation
                self.state.delete_session();
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

    #[rustfmt::skip]
    fn keybinds(&self) -> Line<'_> {
        match self.state.view() {
            View::Normal => {
                let mut items = vec![
                    " Up ".into(), "<K> ".blue().bold(),
                    "Down ".into(), "<J> ".blue().bold(),
                    "Create ".into(), "<N> ".blue().bold(),
                    "Rename ".into(), "<R> ".blue().bold(),
                ];

                if self.state.current_session().is_some_and(|s| s.attached() == 0) {
                    items.extend(vec![
                        "Kill ".into(), "<D> ".blue().bold(),
                    ]);
                }

                items.extend(vec![
                    "Switch ".into(), "<Enter> ".blue().bold(),
                    "Quit ".into(), "<Q> ".blue().bold()
                ]);

                Line::from(items)
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

    fn get_area(&self, area: Rect) -> Rect {
        if self.args.fullscreen() {
            area
        } else {
            area.centered_horizontally(Constraint::Length(80))
                .centered_vertically(Constraint::Length(30))
        }
    }

    fn layout(&self, area: Rect, buf: &mut Buffer) -> (Rect, Rect, Rect) {
        let area = self.get_area(area);

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

        match self.state.view() {
            View::Normal => {
                Paragraph::new(self.state.debug_info()).render(bottom_right_area, buf);
            }
            View::Rename | View::Create => {
                let input_area = Layout::vertical([Constraint::Length(1)])
                    .horizontal_margin(1)
                    .split(bottom_right_area)[0];

                let input =
                    Paragraph::new(self.state.buffer()).style(Style::default().bg(Color::Gray));

                input.render(input_area, buf);
                frame.set_cursor_position(Position::new(
                    input_area.x + self.state.cursor() as u16,
                    input_area.y,
                ));
            }
        }
    }
}
