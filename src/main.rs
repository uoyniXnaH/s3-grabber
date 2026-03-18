mod app;
mod event;
mod ui;

use std::{io, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use event::Event;

fn main() -> io::Result<()> {
    // ── Set up terminal ────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── Run application ────────────────────────────────────────────────────
    let mut app = App::new();
    let tick_rate = Duration::from_millis(250);
    let result = run_app(&mut terminal, &mut app, tick_rate);

    // ── Restore terminal (always, even on error) ───────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(app, frame))?;

        match event::next_event(tick_rate)? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => app.quit(),
                KeyCode::Char('+') => app.increment_counter(),
                KeyCode::Char('-') => app.decrement_counter(),
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}
