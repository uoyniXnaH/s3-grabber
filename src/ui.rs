use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::{
    action::{Focus, WorkTab},
    app::App,
    components,
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let [top, main, bottom] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(12),
        Constraint::Length(3),
    ])
    .areas(area);

    render_top_status(frame, top, app);

    let [left, right] =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).areas(main);
    components::browser::render(frame, left, app);
    render_work_pane(frame, right, app);

    components::status_bar::render(frame, bottom, app);

    if app.ui.show_help {
        components::help_modal::render(frame, area);
    }

    if app.ui.show_connection_settings {
        components::connection_settings::render(frame, area, app);
    }

    if app.ui.show_script_picker {
        components::script_picker::render(frame, area, app);
    }

    if app.ui.confirm_quit {
        render_quit_dialog(frame, area);
    }
}

fn render_top_status(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let focus = match app.ui.focus {
        Focus::Browser => "browser",
        Focus::WorkPane => "work",
    };

    let line = Line::from(format!(
        "Profile: {}  Region: {}  Bucket: {}  Path: {}  Target: {}  Mode: {}  Focus: {}",
        app.display_profile(),
        app.session.region,
        app.session.bucket,
        app.session.path,
        app.display_effective_target(),
        app.session.mode,
        focus,
    ));

    let header =
        Paragraph::new(line).block(Block::default().borders(Borders::ALL).title("S3 Grabber"));
    frame.render_widget(header, area);
}

fn render_work_pane(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let [tabs_area, content_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

    let titles = WorkTab::ALL
        .iter()
        .map(|tab| Line::from(tab.label()))
        .collect::<Vec<_>>();
    let selected = WorkTab::ALL
        .iter()
        .position(|tab| *tab == app.ui.tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(Style::new().cyan().bold())
        .block(Block::default().borders(Borders::ALL).title("Work Pane"));
    frame.render_widget(tabs, tabs_area);

    match app.ui.tab {
        WorkTab::Selection => components::selection::render(frame, content_area, app),
        WorkTab::Details => components::details::render(frame, content_area, app),
        WorkTab::Queue => components::queue::render(frame, content_area, app),
        WorkTab::Logs => render_logs(frame, content_area, app),
    }
}

fn render_logs(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = app
        .logs
        .iter()
        .rev()
        .take(12)
        .map(|entry| Line::from(entry.as_str()))
        .collect::<Vec<_>>();

    if lines.is_empty() {
        lines.push(Line::from("No logs yet."));
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Logs"));
    frame.render_widget(paragraph, area);
}

fn render_quit_dialog(frame: &mut Frame, area: ratatui::layout::Rect) {
    let [_, middle, _] = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(5),
        Constraint::Percentage(40),
    ])
    .areas(area);
    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(60),
        Constraint::Percentage(20),
    ])
    .areas(middle);

    let text = vec![
        Line::from("Downloads are still running. Quit anyway?"),
        Line::from("Press y to confirm, Esc to cancel."),
    ];
    let dialog =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Confirm Quit"));
    frame.render_widget(dialog, center);
}
