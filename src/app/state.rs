use std::{io, path::PathBuf};

use ratatui::widgets::TableState;

use super::{config, config::AppConfig, repo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddingRepo,
    ConfirmDelete,
    ErrorPopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    OpenAdd,
    OpenDeleteConfirm,
    ClosePopup,
    InputChar(char),
    Backspace,
    SubmitAdd,
    ConfirmDelete,
    RefreshOverview,
}

#[derive(Debug)]
pub struct App {
    pub mode: Mode,
    pub cfg_path: PathBuf,
    pub cfg: AppConfig,

    pub table: TableState,

    pub input: String,
    pub message: String,

    pub pr_rows: Vec<crate::github::PrOverviewRow>,
    pub pr_selected: ratatui::widgets::TableState,
}

impl App {
    pub fn new(cfg_path: PathBuf, cfg: AppConfig) -> Self {
        let mut table = TableState::default();
        if !cfg.repos.is_empty() {
            table.select(Some(0));
        }
        let mut pr_selected = TableState::default();
        pr_selected.select(None);
        Self {
            mode: Mode::Normal,
            cfg_path,
            cfg,
            table,
            input: String::new(),
            message: String::new(),
            pr_rows: Vec::new(),
            pr_selected,
        }
    }

    pub fn selected_idx(&self) -> Option<usize> {
        self.table.selected()
    }

    fn select_clamped(&mut self, idx: usize) {
        if self.cfg.repos.is_empty() {
            self.table.select(None);
        } else {
            let i = idx.min(self.cfg.repos.len().saturating_sub(1));
            self.table.select(Some(i));
        }
    }

    pub fn move_up(&mut self) {
        if let Some(i) = self.selected_idx() {
            self.select_clamped(i.saturating_sub(1));
        }
    }

    pub fn move_down(&mut self) {
        if let Some(i) = self.selected_idx() {
            self.select_clamped(i.saturating_add(1));
        } else if !self.cfg.repos.is_empty() {
            self.table.select(Some(0));
        }
    }

    pub fn open_add(&mut self) {
        self.mode = Mode::AddingRepo;
        self.input.clear();
        self.message.clear();
    }

    pub fn open_delete_confirm(&mut self) {
        if self.selected_idx().is_some() {
            self.mode = Mode::ConfirmDelete;
            self.message = "Delete selected repo? (y/n)".into();
        }
    }

    pub fn close_popup(&mut self) {
        self.mode = Mode::Normal;
        self.message.clear();
        self.input.clear();
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.mode = Mode::ErrorPopup;
        self.message = msg.into();
    }

    pub fn try_add_repo(&mut self) -> Result<(), io::Error> {
        match self.cfg.normalize_and_add(&self.input) {
            Ok(added) => {
                if !added {
                    self.set_error("Repo already exists in config");
                    return Ok(());
                }
                config::save(&self.cfg_path, &self.cfg)?;
                // select the added repo
                if let Ok(normalized) = repo::normalize_repo_url(&self.input) {
                    if let Some(pos) = self.cfg.repos.iter().position(|r| r == &normalized) {
                        self.table.select(Some(pos));
                    }
                }
                self.mode = Mode::Normal;
                self.input.clear();
            }
            Err(e) => self.set_error(e),
        }
        Ok(())
    }

    pub fn delete_selected(&mut self) -> Result<(), io::Error> {
        let Some(idx) = self.selected_idx() else {
            self.close_popup();
            return Ok(());
        };

        if !self.cfg.remove_at(idx) {
            self.set_error("Nothing to delete");
            return Ok(());
        }

        config::save(&self.cfg_path, &self.cfg)?;

        if self.cfg.repos.is_empty() {
            self.table.select(None);
        } else {
            self.select_clamped(idx);
        }

        self.mode = Mode::Normal;
        self.message.clear();
        Ok(())
    }

    pub fn refresh_overview(&mut self, gh: &crate::github::GitHubClient) -> Result<(), io::Error> {
        self.pr_rows.clear();

        for repo_url in &self.cfg.repos {
            let (owner, name) = crate::app::repo::owner_and_name(repo_url)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

            let rows = crate::app::overview::fetch_repo_open_prs(gh, &owner, &name)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            self.pr_rows.extend(rows);
        }

        if self.pr_rows.is_empty() {
            self.pr_selected.select(None);
        } else {
            self.pr_selected.select(Some(0));
        }

        Ok(())
    }
}
