#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use std::time::{Duration, Instant};

use crate::{
    App,
    mpd_worker::MpdData,
    utils::{format_duration, render_centered_text, render_progress_bar, render_stretchable_bar},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
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
    ShowVolume,
}

pub struct CassetteWidget<'a> {
    pub current_frame: usize,
    pub status_line_mode: StatusLineMode,
    pub mpd_data: &'a MpdData,
}

impl<'a> CassetteWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        let mode = if let Some(change) = app.last_volume_change {
            if Instant::now().duration_since(change) < Duration::from_secs(1) {
                StatusLineMode::ShowVolume
            } else {
                StatusLineMode::Playing
            }
        } else {
            StatusLineMode::Playing
        };

        Self {
            current_frame: app.frame_number as usize,
            status_line_mode: mode,
            mpd_data: &app.mpd_data,
        }
    }
}

impl Widget for CassetteWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < WIDTH || area.height < HEIGHT {
            return;
        }

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
        .style(Style::default().bold()),
        x,
        y + 2,
    );

    render_centered_text(
        buf,
        Line::from(artist).style(Style::new().red().italic()),
        x,
        y + 3,
    );
}

fn render_spokes(buf: &mut Buffer, x: u16, y: u16, frame_number: usize) {
    let style = Style::default().fg(Color::LightMagenta);
    for (i, line) in REEL_FRAMES[frame_number].iter().enumerate() {
        let dy = y + 4 + i as u16;
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
        StatusLineMode::Playing => Line::from(Span::styled(time_information, Color::Green).bold()),
        StatusLineMode::ShowVolume => {
            let bar = render_progress_bar(data.volume, 100, 20);
            let text = format!("[{bar}]");
            Line::from(Span::styled(text, Color::Green))
        }
    };

    render_centered_text(buf, text.style(Style::new().bold()), x, y + 9);

    let mut status_string = String::new();

    let flags = [
        (data.repeat, "REPEAT"),
        (data.random, "RANDOM"),
        (data.consume, "CONSUME"),
        (data.single, "SINGLE"),
    ];

    for (flag, label) in flags {
        if flag {
            if !status_string.is_empty() {
                status_string.push_str(" | ");
            }
            status_string.push_str(label);
        }
    }

    if status_string.is_empty() {
        status_string.push_str("CLEAR");
        render_centered_text(buf, Line::from("CLEAR").style(Color::Gray), x, y + 10);
        return;
    }

    render_centered_text(buf, Line::from(status_string).style(Color::Red), x, y + 10);
}
