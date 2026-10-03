use ratatui::{layout::Rect, style::Stylize, text::Line, widgets::Paragraph, Frame};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let line = match app.browser.warning.as_deref() {
        Some(message) => Line::from(format!("Warning: {message}")).red(),
        None => Line::from("No warnings".dim()),
    };
    frame.render_widget(Paragraph::new(line), area);
}
