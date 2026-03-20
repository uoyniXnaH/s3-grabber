use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();
    lines.push(Line::from(format!(
        "Selected items: {}",
        app.selected_count()
    )));
    lines.push(Line::from(""));

    if app.browser.selected.is_empty() {
        lines.push(Line::from("No selection yet.".dim().to_string()));
        lines.push(Line::from(
            "Use Space to add/remove current item.".dim().to_string(),
        ));
        lines.push(Line::from(
            "Use a to select all, x to clear.".dim().to_string(),
        ));
    } else {
        for name in &app.browser.selected {
            lines.push(Line::from(format!("- {name}")));
        }
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Selection"));
    frame.render_widget(paragraph, area);
}
