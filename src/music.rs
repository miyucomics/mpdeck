#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

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
}
