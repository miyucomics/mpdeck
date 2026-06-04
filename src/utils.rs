#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::cassette::WIDTH;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Paragraph, Widget},
};

pub fn render_stretchable_bar(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    start: &str,
    end: &str,
    tile: &str,
    style: Style,
) {
    buf.set_string(x, y, start, style);
    for i in 2..(WIDTH - 2) {
        buf.set_string(x + i, y, tile, style);
    }
    buf.set_string(x + WIDTH - 2, y, end, style);
}

pub fn render_centered_text(buf: &mut Buffer, line: Line, x: u16, y: u16) {
    Paragraph::new(line)
        .alignment(Alignment::Center)
        .render(Rect::new(x, y, WIDTH, 1), buf);
}

pub fn format_duration(ms: u32) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

pub fn render_progress_bar(volume: u8, max_volume: u8, width: u8) -> String {
    let filled = (f32::from(volume) / f32::from(max_volume)) * f32::from(width);

    format!(
        "{}{}",
        "■".repeat(filled as usize),
        "□".repeat((width - filled as u8) as usize)
    )
}
