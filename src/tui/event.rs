use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;

pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
    App(AppEvent),
}

pub enum AppEvent {
    Quit,
    TmuxSessions,
}

enum ActorEvent {
    Refetch,
}

pub struct EventHandler {
    tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    actor_tx: mpsc::UnboundedSender<ActorEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (actor_tx, actor_rx) = mpsc::unbounded_channel();
        let actor = EventTask::new(tx.clone(), actor_rx);
        tokio::spawn(async { actor.run().await });
        Self { tx, rx, actor_tx }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx.recv().await.context("Failed to recieve event")
    }

    pub fn send(&self, event: AppEvent) {
        let _ = self.tx.send(Event::App(event));
    }

    pub fn request_refetch(&self) {
        self.send_actor(ActorEvent::Refetch);
    }

    fn send_actor(&self, event: ActorEvent) {
        let _ = self.actor_tx.send(event);
    }
}

struct EventTask(
    mpsc::UnboundedSender<Event>,
    mpsc::UnboundedReceiver<ActorEvent>,
);

impl EventTask {
    fn new(
        sender: mpsc::UnboundedSender<Event>,
        reciever: mpsc::UnboundedReceiver<ActorEvent>,
    ) -> Self {
        Self(sender, reciever)
    }

    const FPS: f64 = 15.0;
    const TMUX_INTERVAL: f64 = 5.0;

    async fn run(mut self) -> Result<()> {
        let mut reader = EventStream::new();

        let tick_rate = Duration::from_secs_f64(1.0 / Self::FPS);
        let mut tick = tokio::time::interval(tick_rate);

        let tmux_tick_rate = Duration::from_secs_f64(Self::TMUX_INTERVAL);
        let mut tmux_tick = tokio::time::interval(tmux_tick_rate);

        loop {
            let tick_delay = tick.tick();
            let tmux_tick_delay = tmux_tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
                _ = self.0.closed() => {
                    break;
                }
                _ = tick_delay => {
                    self.send(Event::Tick);
                }
                _ = tmux_tick_delay => {
                    self.fetch_sessions();
                }
                Some(event) = self.1.recv() => {
                    self.handle_event(event);
                }
                Some(Ok(event)) = crossterm_event => {
                    self.send(Event::Crossterm(event));
                }
            }
        }

        Ok(())
    }

    fn handle_event(&self, event: ActorEvent) {
        match event {
            ActorEvent::Refetch => self.fetch_sessions(),
        }
    }

    fn send(&self, event: Event) {
        let _ = self.0.send(event);
    }

    fn fetch_sessions(&self) {
        let _ = self.0.send(Event::App(AppEvent::TmuxSessions));
    }
}
