use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect) {
    let modal = centered_rect(70, 70, area);
    frame.render_widget(Clear, modal);

    let lines = vec![
        Line::from("Navigation: Up/Down move, Enter open, Backspace parent"),
        Line::from("Selection: Space toggle, a select all, x clear"),
        Line::from("Tabs/Views: Tab next, Shift+Tab previous"),
        Line::from("Operations: d download selected, D download folder, S select script, s run script, r refresh"),
        Line::from("System: h/? toggle help, q quit, y confirm quit, Esc close dialogs"),
    ];

    let panel = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Help (press h or Esc to close)"),
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
