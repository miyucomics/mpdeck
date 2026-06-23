#![warn(clippy::pedantic)]

use crate::{
    music::MpdData,
    utils::{format_duration, render_centered_text, render_progress_bar, render_stretchable_bar},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub const HEIGHT: u16 = 13;
pub const WIDTH: u16 = 46;

const WINDOW_LINES: [&str; 3] = [
    "╔═══╗        ╔═══╗",
    "║ ◎ ║ ╌╌╌╌╌╌ ║ ◎ ║",
    "╚═══╝        ╚═══╝",
];

const TAPE_LINES: [&str; 4] = [
    "╭──────────╮",
    "│▓▓░░░░░░▓▓│",
    "│▓░      ░▓│",
    "╰──────────╯",
];

pub const REEL_FRAMES: [[&str; 5]; 4] = [
    [" .---. ", "/  |  \\", "|  o  |", "\\  |  /", " '---' "],
    [" .---. ", "/   / \\", "|  o  |", "\\ /   /", " '---' "],
    [" .---. ", "/     \\", "|--o--|", "\\     /", " '---' "],
    [" .---. ", "/ \\   \\", "|  o  |", "\\   \\ /", " '---' "],
];

pub enum StatusLineMode {
    Playing,
    PlayingShuffled,
    Nothing,
    ShowVolume,
}

pub struct CassetteWidget<'a> {
    pub current_frame: usize,
    pub status_line_mode: StatusLineMode,
    pub mpd_data: &'a MpdData,
}

impl<'a> CassetteWidget<'a> {
    pub fn new(mpd_data: &'a MpdData, current_frame: usize) -> Self {
        let mode = if mpd_data.show_volume {
            StatusLineMode::ShowVolume
        } else if !mpd_data.playing && mpd_data.current_ms == 0 {
            StatusLineMode::Nothing
        } else if mpd_data.shuffled {
            StatusLineMode::PlayingShuffled
        } else {
            StatusLineMode::Playing
        };

        Self {
            current_frame,
            status_line_mode: mode,
            mpd_data,
        }
    }
}

impl Widget for CassetteWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x = area.left();
        let y = area.top();

        render_frame(buf, x, y);
        render_labels(buf, x, y, &self.mpd_data.title, &self.mpd_data.artist);
        render_spokes(buf, x, y, self.current_frame);
        render_window(buf, x, y);
        render_status(buf, x, y, &self.status_line_mode, self.mpd_data);
    }
}

fn render_frame(buf: &mut Buffer, x: u16, y: u16) {
    let style = Style::new().fg(Color::Cyan);
    render_stretchable_bar(buf, x, y, "╭─", "─╮", "═", style);
    render_stretchable_bar(buf, x, y + 1, "│┈", "┈│", "┈", style);

    for i in 2..(HEIGHT - 2) {
        render_stretchable_bar(buf, x, y + i, "│ ", " │", " ", style);
    }

    render_stretchable_bar(buf, x, y + HEIGHT - 2, "│─", "─│", "─", style);
    render_stretchable_bar(buf, x, y + HEIGHT - 1, "╰─", "─╯", "═", style);

    let screw_style = Color::White;
    for &dx in &[2, WIDTH - 3] {
        buf.set_string(x + dx, y + 1, "x", screw_style);
        buf.set_string(x + dx, y + 10, "x", screw_style);
    }
}

fn render_labels(buf: &mut Buffer, x: u16, y: u16, title: &str, artist: &str) {
    render_centered_text(
        buf,
        Line::from(vec![
            Span::styled(" ★ ", Color::Yellow),
            Span::styled(title, Color::Magenta),
            Span::styled(" ★ ", Color::Yellow),
        ])
        .style(Style::default().bg(Color::Black).bold()),
        x,
        y,
        2,
    );

    let subtitle_style = Color::DarkGray;
    render_centered_text(buf, Line::from(artist).style(subtitle_style), x, y, 3);
    render_centered_text(buf, Line::from("mpdeck").style(subtitle_style), x, y, 10);
}

fn render_spokes(buf: &mut Buffer, x: u16, y: u16, frame_number: usize) {
    let style = Style::default().fg(Color::LightMagenta);
    for (i, line) in REEL_FRAMES[frame_number].iter().enumerate() {
        let dy = y + 4 + u16::try_from(i).unwrap();
        buf.set_string(x + 4, dy, line, style);
        buf.set_string(x + 35, dy, line, style);
    }
}

fn render_window(buf: &mut Buffer, x: u16, y: u16) {
    for (i, line) in WINDOW_LINES.iter().enumerate() {
        buf.set_string(x + 14, y + 4 + i as u16, *line, Color::Cyan);
    }

    for (i, line) in TAPE_LINES.iter().enumerate() {
        buf.set_string(x + 17, y + 5 + i as u16, *line, Color::Yellow);
    }
}

fn render_status(buf: &mut Buffer, x: u16, y: u16, mode: &StatusLineMode, data: &MpdData) {
    let time_information = format!(
        "{}/{}",
        format_duration(data.current_ms),
        format_duration(data.duration_ms)
    );

    let text = match mode {
        StatusLineMode::Playing => Line::from(Span::styled(time_information, Color::Green)),
        StatusLineMode::PlayingShuffled => Line::from(vec![
            Span::styled(">< ", Color::Yellow),
            Span::styled(time_information, Color::Green),
            Span::styled("   ", Color::Yellow),
        ]),
        StatusLineMode::Nothing => Line::from(Span::styled("● READY", Color::Green)),
        StatusLineMode::ShowVolume => {
            let bar = render_progress_bar(data.volume, data.volume_max, 10);
            let text = format!("[{bar}]");
            Line::from(Span::styled(text, Color::Green))
        }
    };

    render_centered_text(buf, text.style(Style::new().bold()), x, y, 9);
}
