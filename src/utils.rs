use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::cassette::WIDTH;

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

pub fn render_centered_text(buf: &mut Buffer, line: Line, x: u16, y: u16, relative: u16) {
    Paragraph::new(line)
        .alignment(Alignment::Center)
        .render(Rect::new(x, y + relative, WIDTH, 1), buf);
}

pub fn format_duration(ms: i32) -> String {
    if ms <= 0 {
        return "0:00".to_string();
    }
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

pub fn render_progress_bar(volume: i8, max_volume: i8, width: i32) -> String {
    let filled = ((volume as f64 / max_volume as f64) * width as f64) as usize;

    format!(
        "{}{}",
        "■".repeat(filled),
        "□".repeat(width as usize - filled)
    )
}
