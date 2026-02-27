use std::io;

pub mod config;
pub mod event;
pub mod overview;
pub mod repo;
pub mod state;
pub mod ui;

pub use state::App;

use crate::tui::TuiTerminal;

pub fn run(terminal: &mut TuiTerminal, app: &mut App) -> Result<(), io::Error> {
    // GitHub client (PAT in .env as GITHUB_TOKEN)
    let gh = crate::github::GitHubClient::from_env()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Initial fetch so the overview is populated right away
    app.refresh_overview(&gh)?;

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !event::poll(std::time::Duration::from_millis(250))? {
            continue;
        }

        if let Some(action) = event::read_action(app.mode)? {
            match action {
                state::Action::Quit => return Ok(()),

                state::Action::MoveUp => app.move_up(),
                state::Action::MoveDown => app.move_down(),

                state::Action::OpenAdd => app.open_add(),
                state::Action::OpenDeleteConfirm => app.open_delete_confirm(),
                state::Action::ClosePopup => app.close_popup(),

                state::Action::InputChar(c) => app.input.push(c),
                state::Action::Backspace => {
                    app.input.pop();
                }

                state::Action::SubmitAdd => {
                    // Save config (repo list)
                    app.try_add_repo()?;
                    // Refresh overview (best effort; show error popup on failure)
                    app.refresh_overview(&gh)?;
                }

                state::Action::ConfirmDelete => {
                    // Save config (repo list)
                    app.delete_selected()?;
                    // Refresh overview (best effort; show error popup on failure)
                    app.refresh_overview(&gh)?;
                }

                state::Action::RefreshOverview => {
                    app.refresh_overview(&gh)?;
                }
            }
        }
    }
}
