use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let current = app.browser.items.get(app.browser.cursor);
    let lines = if let Some(item) = current {
        if item.is_dir {
            vec![
                Line::from("Directory selected."),
                Line::from(
                    "Preview is available for text objects only."
                        .dim()
                        .to_string(),
                ),
            ]
        } else {
            vec![
                Line::from(format!("Object: {}", item.name)),
                Line::from("---"),
                Line::from("This is a scaffold preview panel.".dim().to_string()),
                Line::from(
                    "Actual object fetch/preview will be implemented next."
                        .dim()
                        .to_string(),
                ),
            ]
        }
    } else {
        vec![Line::from("No object selected.")]
    };

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Preview"));
    frame.render_widget(paragraph, area);
}
