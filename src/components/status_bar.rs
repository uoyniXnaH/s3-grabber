use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from(
            "Keys: h help  c connection  S scripts  q quit  ↑↓ move  Enter open  Backspace up  Space select  a all  x clear  d download  l logs  s run-script"
                .dim()
                .to_string(),
        ),
        Line::from(format!(
            "Progress: {}/{} files  {}/{} bytes  {:.1}MB/s  ETA {}",
            app.queue.done_files,
            app.queue.total_files,
            app.queue.done_bytes,
            app.queue.total_bytes,
            app.queue.speed_mbps,
            app.queue.eta
        )),
        Line::from(format!(
            "Profile: {}  Region: {}  Bucket: {}  Target: {}  Script: {}  Mode: {}  Last: {}",
            app.display_profile(),
            app.session.region,
            app.session.bucket,
            app.display_effective_target(),
            app.selected_script_label(),
            app.script_mode_label(),
            app.script.last_result
        )),
    ];

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}
