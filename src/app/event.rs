use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use super::state::{Action, Mode};

pub fn poll(timeout: std::time::Duration) -> io::Result<bool> {
    event::poll(timeout)
}

pub fn read_action(mode: Mode) -> io::Result<Option<Action>> {
    let ev = event::read()?;
    let Event::Key(k) = ev else { return Ok(None) };
    if k.kind != KeyEventKind::Press {
        return Ok(None);
    }

    // Global quit
    if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(Some(Action::Quit));
    }

    let action = match mode {
        Mode::Normal => match k.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
            KeyCode::Char('a') => Some(Action::OpenAdd),
            KeyCode::Char('d') => Some(Action::OpenDeleteConfirm),
            KeyCode::Char('r') => Some(Action::RefreshOverview),
            _ => None,
        },

        Mode::AddingRepo => match k.code {
            KeyCode::Esc => Some(Action::ClosePopup),
            KeyCode::Enter => Some(Action::SubmitAdd),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Char(c) => {
                if c.is_control() {
                    None
                } else {
                    Some(Action::InputChar(c))
                }
            }
            _ => None,
        },

        Mode::ConfirmDelete => match k.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(Action::ConfirmDelete),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::ClosePopup),
            _ => None,
        },

        Mode::ErrorPopup => match k.code {
            KeyCode::Enter | KeyCode::Esc => Some(Action::ClosePopup),
            _ => None,
        },
    };

    Ok(action)
}
