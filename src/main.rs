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
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    widgets::{Block, BorderType},
};
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
    last_volume_change: Option<Instant>,
    command_tx: Sender<MpdCommand>,
    show_queue: bool,
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
            last_volume_change: None,
            command_tx,
            show_queue: false,
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
            KeyCode::Char('m') => self.show_queue = !self.show_queue,
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
            KeyCode::Char('u') => {
                let _ = self.command_tx.send(MpdCommand::IncreaseVolume);
                self.last_volume_change = Some(Instant::now());
            }
            KeyCode::Char('i') => {
                let _ = self.command_tx.send(MpdCommand::DecreaseVolume);
                self.last_volume_change = Some(Instant::now());
            }
            _ => {}
        }
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let (mut app, rx) = App::new();

    let cassette_width = crate::cassette::WIDTH;
    let cassette_height = crate::cassette::HEIGHT;
    let queue_width = crate::cassette::WIDTH;

    loop {
        while let Ok(latest_mpd_data) = rx.try_recv() {
            app.mpd_data = latest_mpd_data;
        }

        app.tick();

        terminal.draw(|frame| {
            if app.show_queue {
                let total_width = cassette_width + 1 + queue_width;

                let centered_area = frame.area().centered(
                    Constraint::Length(total_width),
                    Constraint::Length(cassette_height),
                );

                let rects = Layout::horizontal([
                    Constraint::Length(queue_width),
                    Constraint::Length(1),
                    Constraint::Length(cassette_width),
                ])
                .split(centered_area);

                frame.render_widget(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(" Queue "),
                    rects[0],
                );

                frame.render_widget(CassetteWidget::new(&app), rects[2]);
            } else {
                frame.render_widget(
                    CassetteWidget::new(&app),
                    frame.area().centered(
                        Constraint::Length(cassette_width),
                        Constraint::Length(cassette_height),
                    ),
                );
            }
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
