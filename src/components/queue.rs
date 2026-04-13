use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, QueueJobStatus};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![
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
        Line::from(""),
    ];

    for job in app.queue.jobs.iter().take(8) {
        let status = match job.status {
            QueueJobStatus::Pending => "PENDING".dim().to_string(),
            QueueJobStatus::Running => "RUNNING".cyan().bold().to_string(),
            QueueJobStatus::Done => "DONE".green().to_string(),
            QueueJobStatus::Failed => "FAILED".red().to_string(),
        };
        lines.push(Line::from(format!(
            "{}  a:{:>2}  {}",
            status, job.attempts, job.key
        )));
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Queue"));
    frame.render_widget(paragraph, area);
}
