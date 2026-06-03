#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use mpd::Client;

#[derive(Debug, Default)]
pub struct MpdData {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub show_volume: bool,

    pub current_ms: u32,
    pub duration_ms: u32,
    pub volume: u8,

    pub repeat: bool,
    pub random: bool,
    pub consume: bool,
    pub single: bool,
}

impl MpdData {
    pub fn new() -> MpdData {
        MpdData {
            title: String::new(),
            artist: String::new(),
            playing: true,
            show_volume: false,

            current_ms: 87000,
            duration_ms: 232_000,
            volume: 70,

            random: false,
            repeat: false,
            consume: false,
            single: false,
        }
    }
}

pub enum MpdCommand {
    TogglePause,
    PreviousTrack,
    NextTrack,
    ToggleRepeat,
    ToggleRandom,
    ToggleConsume,
    ToggleSingle,
}

pub fn construct_worker_thread() -> (Sender<MpdCommand>, Receiver<MpdData>) {
    let (data_tx, data_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut client = Client::connect("127.0.0.1:6600").unwrap();

        loop {
            if let Ok(status) = client.status() {
                while let Ok(command) = command_rx.try_recv() {
                    let _ = match command {
                        MpdCommand::TogglePause => client.toggle_pause(),
                        MpdCommand::PreviousTrack => client.prev(),
                        MpdCommand::NextTrack => client.next(),
                        MpdCommand::ToggleRepeat => client.repeat(!status.repeat),
                        MpdCommand::ToggleRandom => client.random(!status.random),
                        MpdCommand::ToggleConsume => client.consume(!status.consume),
                        MpdCommand::ToggleSingle => client.single(!status.single),
                    };
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
                    fresh_data.title = song.title.unwrap_or_else(|| "Untitled Title".to_string());
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

    (command_tx, data_rx)
}
