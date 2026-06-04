#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::struct_excessive_bools)]

use mpd::{Client, Idle, Subsystem};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

const RELEVANT_SUBSYSTEMS: &[Subsystem] =
    &[Subsystem::Player, Subsystem::Mixer, Subsystem::Options];

#[derive(Debug, Default)]
pub struct MpdData {
    pub title: String,
    pub artist: String,
    pub playing: bool,

    pub current_ms: u32,
    pub duration_ms: u32,
    pub volume: u8,

    pub repeat: bool,
    pub random: bool,
    pub consume: bool,
    pub single: bool,
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
        let mut client = Client::connect("127.0.0.1:6600").expect("Commander failed to connect.");

        while let Ok(command) = command_rx.recv() {
            if let Ok(status) = client.status() {
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
    });

    thread::spawn(move || {
        let create_sync_packet = |client: &mut Client| -> Option<MpdData> {
            let status = client.status().ok()?;

            let mut packet = MpdData {
                title: "Untitled Title".to_string(),
                artist: "Untitled Artist".to_string(),
                playing: status.state == mpd::State::Play,
                current_ms: 0,
                duration_ms: 0,
                volume: status.volume.cast_unsigned(),
                repeat: status.repeat,
                random: status.random,
                consume: status.consume,
                single: status.single,
            };

            if let Ok(Some(song)) = client.currentsong() {
                packet.title = song.title.unwrap_or_else(|| "Untitled Title".to_string());
                packet.artist = song.artist.unwrap_or_else(|| "Untitled Artist".to_string());
            }

            if let Some(elapsed) = status.elapsed {
                packet.current_ms = elapsed.as_millis() as u32;
            }

            if let Some(duration) = status.duration {
                packet.duration_ms = duration.as_millis() as u32;
            }

            Some(packet)
        };

        let mut client = Client::connect("127.0.0.1:6600").expect("Watcher failed to connect.");
        if let Some(initial_data) = create_sync_packet(&mut client) {
            let _ = data_tx.send(initial_data);
        }

        loop {
            if let Ok(_events) = client.wait(RELEVANT_SUBSYSTEMS)
                && let Some(data) = create_sync_packet(&mut client)
            {
                if data_tx.send(data).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    (command_tx, data_rx)
}
