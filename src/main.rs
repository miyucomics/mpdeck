#![warn(clippy::pedantic)]

mod cassette;
mod music;
mod utils;

use crate::{
    cassette::{CassetteWidget, CassetteWidgetState, StatusLineMode},
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
    pub client: Client,
    pub cassette_state: CassetteWidgetState,
    pub tick_rate: Duration,
    pub last_tick: Instant,
    pub accumulated_time: Duration,
    pub should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            client: Client::connect("127.0.0.1:6600").unwrap(),
            cassette_state: CassetteWidgetState {
                title: String::new(),
                artist: String::new(),
                current_frame: 0,
                mode: StatusLineMode::Playing,
                data: MpdData::new(),
            },
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

        if self.cassette_state.data.playing {
            self.accumulated_time += delta;
        }

        while self.accumulated_time >= self.tick_rate {
            if self.cassette_state.data.playing {
                self.cassette_state.next_frame();
            }
            self.accumulated_time -= self.tick_rate;
        }

        if let Ok(status) = self.client.status() {
            self.cassette_state.data.playing = status.state == mpd::State::Play;
            if let Some(elapsed) = status.elapsed {
                self.cassette_state.data.current_ms = i32::try_from(elapsed.as_millis()).unwrap();
            }

            if let Ok(Some(song)) = self.client.currentsong() {
                self.cassette_state.title = song.title.unwrap();
                self.cassette_state.artist = song.artist.unwrap();
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
                CassetteWidget {
                    state: &app.cassette_state,
                },
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
