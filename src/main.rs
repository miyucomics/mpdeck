#![warn(clippy::pedantic)]

mod cassette;
mod music;
mod utils;

use crate::{
    cassette::{CassetteWidget, REEL_FRAMES},
    music::MpdData,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use mpd::Client;
use ratatui::{DefaultTerminal, layout::Constraint};
use std::{
    io::Error,
    time::{Duration, Instant},
};

fn main() -> Result<(), Error> {
    ratatui::run(app)?;
    Ok(())
}

pub struct App {
    client: Client,
    frame_number: u8,
    mpd_data: MpdData,
    tick_rate: Duration,
    last_tick: Instant,
    accumulated_time: Duration,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            client: Client::connect("127.0.0.1:6600").unwrap(),
            frame_number: 0,
            mpd_data: MpdData::new(),
            tick_rate: Duration::from_millis(150),
            last_tick: Instant::now(),
            accumulated_time: Duration::from_secs(0),
            should_quit: false,
        }
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

        if let Ok(status) = self.client.status() {
            self.mpd_data.playing = status.state == mpd::State::Play;
            if let Some(elapsed) = status.elapsed {
                self.mpd_data.current_ms = i32::try_from(elapsed.as_millis()).unwrap();
            }

            if let Ok(Some(song)) = self.client.currentsong() {
                self.mpd_data.title = song.title.unwrap();
                self.mpd_data.artist = song.artist.unwrap();
            }
        }
    }

    fn handle_event(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(' ') => self.client.toggle_pause().unwrap(),
            _ => {}
        }
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    loop {
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
