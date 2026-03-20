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
        (KeyCode::Backspace, _) => Action::GoParent,
        (KeyCode::Tab, KeyModifiers::NONE) => Action::NextTab,
        (KeyCode::BackTab, _) => Action::PreviousTab,
        (KeyCode::Esc, _) => Action::CancelDialog,
        (KeyCode::Char(' '), _) => Action::ToggleSelectCurrent,
        (KeyCode::Char('/'), _) => Action::OpenFilter,
        (KeyCode::Char('a'), _) => Action::SelectAllVisible,
        (KeyCode::Char('x'), _) => Action::ClearSelection,
        (KeyCode::Char('f'), _) => Action::FocusNext,
        (KeyCode::Char('p'), _) => Action::OpenPreview,
        (KeyCode::Char('l'), _) => Action::OpenLogsTab,
        (KeyCode::Char('d'), _) => Action::QueueDownloadSelected,
        (KeyCode::Char('D'), _) => Action::QueueDownloadFolder,
        (KeyCode::Char('s'), _) => Action::RunScript,
        (KeyCode::Char('r'), _) => Action::Refresh,
        (KeyCode::Char('h'), _) | (KeyCode::Char('?'), _) => Action::ToggleHelp,
        (KeyCode::Char('q'), _) => Action::QuitRequested,
        (KeyCode::Char('y'), _) => Action::ConfirmQuit,
        _ => Action::Tick,
    }
}
