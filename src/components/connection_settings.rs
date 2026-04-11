use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, ConnectionField};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let modal = centered_rect(72, 75, area);
    frame.render_widget(Clear, modal);

    let mut lines = Vec::new();
    lines.push(Line::from(
        "Configure S3 context. Profile is optional (empty => default-chain).",
    ));
    lines.push(Line::from(
        "Endpoint URL is optional (empty => standard AWS S3 endpoint).",
    ));
    lines.push(Line::from(
        "Tab/Shift+Tab move field  Enter apply  Esc cancel",
    ));
    lines.push(Line::from(""));

    push_field(
        &mut lines,
        ConnectionField::Profile,
        app,
        &app.ui.connection_draft.profile,
    );
    push_field(
        &mut lines,
        ConnectionField::Region,
        app,
        &app.ui.connection_draft.region,
    );
    push_field(
        &mut lines,
        ConnectionField::Bucket,
        app,
        &app.ui.connection_draft.bucket,
    );
    push_field(
        &mut lines,
        ConnectionField::Prefix,
        app,
        &app.ui.connection_draft.prefix,
    );
    push_field(
        &mut lines,
        ConnectionField::EndpointUrl,
        app,
        &app.ui.connection_draft.endpoint_url,
    );

    if let Some(warning) = app.connection_modal_warning() {
        lines.push(Line::from(""));
        lines.push(Line::from(format!("Warning: {warning}")).yellow());
    }

    if let Some(error) = &app.ui.connection_draft.error {
        lines.push(Line::from(""));
        lines.push(Line::from(format!("Error: {error}")).red());
    }

    let panel = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Connection Settings"),
    );
    frame.render_widget(panel, modal);
}

fn push_field(lines: &mut Vec<Line<'static>>, field: ConnectionField, app: &App, value: &str) {
    let active = app.ui.connection_draft.active_field == field;
    let marker = if active { ">" } else { " " };
    let text = if value.is_empty() { "<empty>" } else { value };
    let line = Line::from(format!("{marker} {:<18}: {text}", field.label()));
    if active {
        lines.push(line.cyan().bold());
    } else {
        lines.push(line);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, center, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);

    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(center);

    center
}
