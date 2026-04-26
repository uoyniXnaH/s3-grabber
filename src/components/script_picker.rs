use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let modal = centered_rect(72, 72, area);
    frame.render_widget(Clear, modal);

    let mut lines = vec![
        Line::from(format!("Script Dir: {}", app.script.script_dir.display())),
        Line::from(format!(
            "Mode: {} (press m to toggle)",
            app.script_mode_label()
        )),
        Line::from(
            "Enter to select script, r to rescan, Esc to close"
                .dim()
                .to_string(),
        ),
        Line::from(""),
    ];

    if app.script.available_scripts.is_empty() {
        lines.push(Line::from(
            "No scripts found in directory.".yellow().to_string(),
        ));
    } else {
        for (index, script) in app.script.available_scripts.iter().enumerate().take(16) {
            let selected = index == app.ui.script_picker_cursor;
            let marker = if selected { ">" } else { " " };
            let active = if *script == app.script.command {
                "*"
            } else {
                " "
            };
            let text = format!("{}{} {}", marker, active, script);
            if selected {
                lines.push(Line::from(text.cyan().bold().to_string()));
            } else {
                lines.push(Line::from(text));
            }
        }
    }

    if let Some(err) = &app.ui.script_picker_error {
        lines.push(Line::from(""));
        lines.push(Line::from(format!("Error: {err}").red().to_string()));
    }

    let panel = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Script Picker (S)"),
    );
    frame.render_widget(panel, modal);
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
