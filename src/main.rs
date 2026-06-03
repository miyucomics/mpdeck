#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

mod cassette;
mod music;
mod utils;

use crate::{
    cassette::{CassetteWidget, REEL_FRAMES},
    music::{MpdCommand, MpdData},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use mpd::Client;
use ratatui::{DefaultTerminal, layout::Constraint};
use std::{
    io::Error,
    sync::mpsc::{self, Receiver, Sender},
    thread,
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
        let (data_tx, data_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();

        thread::spawn(move || {
            let mut client = Client::connect("127.0.0.1:6600").unwrap();

            loop {
                while let Ok(command) = command_rx.try_recv() {
                    match command {
                        MpdCommand::TogglePause => {
                            let _ = client.toggle_pause();
                        }
                        MpdCommand::PreviousTrack => {
                            let _ = client.prev();
                        }
                        MpdCommand::NextTrack => {
                            let _ = client.next();
                        }
                    }
                }

                let mut fresh_data = MpdData::new();
                if let Ok(status) = client.status() {
                    fresh_data.playing = status.state == mpd::State::Play;

                    if let Some(elapsed) = status.elapsed {
                        fresh_data.current_ms = elapsed.as_millis() as u32;
                    }

                    if let Some(duration) = status.duration {
                        fresh_data.duration_ms = duration.as_millis() as u32;
                    }

                    fresh_data.volume = status.volume.cast_unsigned();

                    fresh_data.repeat = status.repeat;
                    fresh_data.random = status.random;
                    fresh_data.consume = status.consume;
                    fresh_data.single = status.single;

                    if let Ok(Some(song)) = client.currentsong() {
                        fresh_data.title =
                            song.title.unwrap_or_else(|| "Untitled Title".to_string());
                        fresh_data.artist =
                            song.artist.unwrap_or_else(|| "Untitled Artist".to_string());
                    }
                }

                if data_tx.send(fresh_data).is_err() {
                    break;
                }

                thread::sleep(Duration::from_millis(200));
            }
        });

        let app = Self {
            frame_number: 0,
            mpd_data: MpdData::new(),
            tick_rate: Duration::from_millis(150),
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
            self.accumulated_time += delta;
        }

        while self.accumulated_time >= self.tick_rate {
            if self.mpd_data.playing {
                self.frame_number = (self.frame_number + 1) % REEL_FRAMES.len() as u8;
            }
            self.accumulated_time -= self.tick_rate;
        }
    }

    fn handle_event(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(' ') => {
                let _ = self.command_tx.send(MpdCommand::TogglePause);
            }
            KeyCode::Char('j') => {
                let _ = self.command_tx.send(MpdCommand::NextTrack);
            }
            KeyCode::Char('k') => {
                let _ = self.command_tx.send(MpdCommand::PreviousTrack);
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
