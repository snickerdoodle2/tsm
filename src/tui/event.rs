use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;

use crate::TmuxSession;

pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
    App(AppEvent),
}

pub enum AppEvent {
    Quit,
    TmuxSessions(Option<Vec<TmuxSession>>),
}

pub struct EventHandler {
    tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let actor = EventTask::new(tx.clone());
        let tmux_actor = TmuxTask::new(tx.clone());
        tokio::spawn(async { tokio::join!(actor.run(), tmux_actor.run()) });
        Self { tx, rx }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx.recv().await.context("Failed to recieve event")
    }

    pub fn send(&self, event: AppEvent) {
        let _ = self.tx.send(Event::App(event));
    }
}

struct TmuxTask(mpsc::UnboundedSender<Event>);

impl TmuxTask {
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self(sender)
    }

    const DELAY: f64 = 5.0;

    async fn run(self) -> Result<()> {
        let tick_rate = Duration::from_secs_f64(Self::DELAY);
        let mut tick = tokio::time::interval(tick_rate);

        loop {
            let tick_delay = tick.tick();
            tokio::select! {
                _ = self.0.closed() => {
                    break;
                }
                _ = tick_delay => {
                    self.fetch_sessions();
                }
            }
        }

        Ok(())
    }

    fn fetch_sessions(&self) {
        let res = TmuxSession::list().ok();
        let _ = self.0.send(Event::App(AppEvent::TmuxSessions(res)));
    }
}

struct EventTask(mpsc::UnboundedSender<Event>);

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self(sender)
    }

    const FPS: f64 = 15.0;

    async fn run(self) -> Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / Self::FPS);
        let mut reader = EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);

        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
                _ = self.0.closed() => {
                    break;
                }
                _ = tick_delay => {
                    self.send(Event::Tick);
                }
                Some(Ok(event)) = crossterm_event => {
                    self.send(Event::Crossterm(event));
                }
            }
        }

        Ok(())
    }

    fn send(&self, event: Event) {
        let _ = self.0.send(event);
    }
}
