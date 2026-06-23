#![warn(clippy::pedantic)]

#[derive(Debug, Default)]
pub struct MpdData {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub show_volume: bool,
    pub current_ms: i32,
    pub duration_ms: i32,
    pub volume: i8,
    pub volume_max: i8,
    pub shuffled: bool,
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
            volume_max: 100,
            shuffled: true,
        }
    }
}
