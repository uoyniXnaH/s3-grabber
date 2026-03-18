use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// Events produced by the terminal.
#[derive(Debug)]
pub enum Event {
    /// A key was pressed.
    Key(KeyEvent),
    /// No event occurred within the tick interval.
    Tick,
}

/// Block until the next event or until `tick_rate` elapses.
pub fn next_event(tick_rate: Duration) -> std::io::Result<Event> {
    if event::poll(tick_rate)? {
        match event::read()? {
            CrosstermEvent::Key(key) => Ok(Event::Key(key)),
            _ => Ok(Event::Tick),
        }
    } else {
        Ok(Event::Tick)
    }
}
