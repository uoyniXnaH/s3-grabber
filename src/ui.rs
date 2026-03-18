use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

/// Render the full UI for one frame.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // ── Title bar ──────────────────────────────────────────────────────────
    let title = Paragraph::new("s3-grabber")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // ── Main content ───────────────────────────────────────────────────────
    let counter_text = Line::from(vec![
        Span::raw("Counter: "),
        Span::styled(
            app.counter.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let body = Paragraph::new(counter_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Content"));
    frame.render_widget(body, chunks[1]);

    // ── Status / help bar ──────────────────────────────────────────────────
    let help =
        Paragraph::new("Press 'q' to quit  |  '+' increment  |  '-' decrement")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}
