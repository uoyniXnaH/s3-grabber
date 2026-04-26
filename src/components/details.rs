use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, BrowserItemKind};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let current = app.browser.items.get(app.browser.cursor);

    let mut lines = Vec::new();
    match current {
        Some(item) if item.kind == BrowserItemKind::Obj => {
            lines.push(Line::from(format!("Key: {}", item.key)));
            lines.push(Line::from(format!("Name: {}", item.name)));
            lines.push(Line::from(format!(
                "Size: {} bytes",
                item.size.unwrap_or(0)
            )));
            lines.push(Line::from(format!("Modified: {}", item.modified)));
            lines.push(Line::from(format!(
                "Download Path: {}",
                app.download_root
                    .join(item.key.replace('/', std::path::MAIN_SEPARATOR_STR))
                    .display()
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Text preview is intentionally disabled. Use your editor after download."
                    .dim()
                    .to_string(),
            ));
        }
        Some(item) if item.kind == BrowserItemKind::Dir => {
            lines.push(Line::from(format!("Directory: {}", item.key)));
            lines.push(Line::from(
                "Use d or D to queue downloads.".dim().to_string(),
            ));
        }
        Some(item) if item.kind == BrowserItemKind::Parent => {
            lines.push(Line::from("Parent entry selected."));
            lines.push(Line::from("Press Enter to navigate up.".dim().to_string()));
        }
        _ => {
            lines.push(Line::from("No item selected."));
        }
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(paragraph, area);
}
