use std::time::Duration;

use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};

use crate::action::Action;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Event {
    Action(Action),
    Tick,
}

pub fn next_event(tick_rate: Duration) -> std::io::Result<Event> {
    if event::poll(tick_rate)? {
        match event::read()? {
            CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                Ok(Event::Action(map_key_to_action(key)))
            }
            _ => Ok(Event::Tick),
        }
    } else {
        Ok(Event::Tick)
    }
}

fn map_key_to_action(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Up, _) => Action::MoveUp,
        (KeyCode::Down, _) => Action::MoveDown,
        (KeyCode::Left, _) => Action::MoveLeft,
        (KeyCode::Right, _) => Action::MoveRight,
        (KeyCode::Enter, _) => Action::Enter,
        (KeyCode::Backspace, _) => Action::BackspaceKey,
        (KeyCode::Tab, KeyModifiers::NONE) => Action::NextTab,
        (KeyCode::BackTab, _) => Action::PreviousTab,
        (KeyCode::Esc, _) => Action::CancelDialog,
        (KeyCode::Char(' '), _) => Action::ToggleSelectCurrent,
        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => Action::InputChar(ch),
        _ => Action::Tick,
    }
}
