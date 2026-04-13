use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
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

    if let Some(warning) = &app.browser.warning {
        let [warning_area, list_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);
        let warning = Paragraph::new(Line::from(format!("Warning: {warning}")).red())
            .block(Block::default().borders(Borders::ALL).title("S3 Warning"));
        frame.render_widget(warning, warning_area);
        frame.render_stateful_widget(list, list_area, &mut state);
    } else {
        frame.render_stateful_widget(list, area, &mut state);
    }
}
