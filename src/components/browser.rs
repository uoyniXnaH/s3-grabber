use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::{
    action::Focus,
    app::{App, BrowserItemKind},
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.ui.focus == Focus::Browser {
        "S3 Browser [focus]"
    } else {
        "S3 Browser"
    };

    if !app.session.connected {
        let prompt = Paragraph::new(Text::from(vec![
            Line::from("No S3 connection configured".bold()),
            Line::from(""),
            Line::from("Press c to select an AWS profile or endpoint and bucket."),
            Line::from("The browser will list S3 contents after connecting."),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(prompt, area);
        return;
    }

    let items = app
        .browser
        .items
        .iter()
        .map(|item| {
            let selectable = !matches!(item.kind, BrowserItemKind::Parent);
            let mark = if selectable && app.browser.selected.contains(&item.key) {
                "[x]"
            } else if selectable {
                "[ ]"
            } else {
                "   "
            };
            let kind = match item.kind {
                BrowserItemKind::Parent => "UP",
                BrowserItemKind::Dir => "DIR",
                BrowserItemKind::Obj => "OBJ",
            };
            let size = item
                .size
                .map(|v| format!("{v} B"))
                .unwrap_or_else(|| "-".to_string());

            ListItem::new(Line::from(format!(
                "{mark} {kind:<3} {:<26} {:>10} {}",
                item.name, size, item.modified
            )))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default().with_selected(Some(app.browser.cursor));
    let list = List::new(items)
        .highlight_symbol("> ")
        .highlight_style(Style::new().cyan().bold())
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_stateful_widget(list, area, &mut state);
}
