#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

mod cassette;
mod mpd_worker;
mod utils;

use crate::{
    cassette::{CassetteWidget, REEL_FRAMES},
    mpd_worker::{MpdCommand, MpdData, construct_worker_thread},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, layout::Constraint};
use std::{
    io::Error,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

fn main() -> Result<(), Error> {
    ratatui::run(app)?;
    Ok(())
}

pub struct App {
    frame_number: u8,
    mpd_data: MpdData,
    tick_rate: Duration,
    last_tick: Instant,
    accumulated_time: Duration,
    should_quit: bool,
    command_tx: Sender<MpdCommand>,
}

impl App {
    fn new() -> (Self, Receiver<MpdData>) {
        let (command_tx, data_rx) = construct_worker_thread();

        let app = Self {
            frame_number: 0,
            mpd_data: MpdData::default(),
            tick_rate: Duration::from_millis(200),
            last_tick: Instant::now(),
            accumulated_time: Duration::from_secs(0),
            should_quit: false,
            command_tx,
        };

        (app, data_rx)
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick);
        self.last_tick = now;

        if self.mpd_data.playing {
            self.mpd_data.current_ms = self
                .mpd_data
                .current_ms
                .saturating_add(delta.as_millis() as u32);

            if self.mpd_data.current_ms > self.mpd_data.duration_ms {
                self.mpd_data.current_ms = self.mpd_data.duration_ms;
            }

            self.accumulated_time += delta;
            while self.accumulated_time >= self.tick_rate {
                self.frame_number = (self.frame_number + 1) % REEL_FRAMES.len() as u8;
                self.accumulated_time -= self.tick_rate;
            }
        }
    }

    fn handle_event(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(' ' | 'p') => {
                let _ = self.command_tx.send(MpdCommand::TogglePause);
            }
            KeyCode::Char('j') => {
                let _ = self.command_tx.send(MpdCommand::NextTrack);
            }
            KeyCode::Char('k') => {
                let _ = self.command_tx.send(MpdCommand::PreviousTrack);
            }
            KeyCode::Char('z') => {
                let _ = self.command_tx.send(MpdCommand::ToggleRepeat);
            }
            KeyCode::Char('x') => {
                let _ = self.command_tx.send(MpdCommand::ToggleRandom);
            }
            KeyCode::Char('c') => {
                let _ = self.command_tx.send(MpdCommand::ToggleConsume);
            }
            KeyCode::Char('v') => {
                let _ = self.command_tx.send(MpdCommand::ToggleSingle);
            }
            _ => {}
        }
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let (mut app, rx) = App::new();

    loop {
        while let Ok(latest_mpd_data) = rx.try_recv() {
            app.mpd_data = latest_mpd_data;
        }

        app.tick();

        terminal.draw(|frame| {
            frame.render_widget(
                CassetteWidget::new(&app.mpd_data, app.frame_number as usize),
                frame.area().centered(
                    Constraint::Length(crate::cassette::WIDTH),
                    Constraint::Length(crate::cassette::HEIGHT),
                ),
            );
        })?;

        if event::poll(Duration::from_millis(10))?
            && let Event::Key(key) = &event::read()?
        {
            app.handle_event(key);
        }

        if app.should_quit {
            break Ok(());
        }
    }
}
