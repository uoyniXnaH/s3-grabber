use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from(format!(
            "Files: {}/{}",
            app.queue.done_files, app.queue.total_files
        )),
        Line::from(format!(
            "Bytes: {}/{}",
            app.queue.done_bytes, app.queue.total_bytes
        )),
        Line::from(format!("Speed: {:.1} MB/s", app.queue.speed_mbps)),
        Line::from(format!("ETA: {}", app.queue.eta)),
    ];

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Queue"));
    frame.render_widget(paragraph, area);
}
