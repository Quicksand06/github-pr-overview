use std::{fs, io, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub repos: Vec<String>,
}

impl AppConfig {
    pub fn normalize_and_add(&mut self, repo_url: &str) -> Result<bool, String> {
        let normalized = crate::app::repo::normalize_repo_url(repo_url)?;
        if self.repos.iter().any(|r| r == &normalized) {
            return Ok(false);
        }
        self.repos.push(normalized);
        self.repos.sort();
        Ok(true)
    }

    pub fn remove_at(&mut self, idx: usize) -> bool {
        if idx >= self.repos.len() {
            return false;
        }
        self.repos.remove(idx);
        true
    }
}

pub fn default_config_path() -> io::Result<PathBuf> {
    let base = if let Some(dir) = dirs::config_dir() {
        dir
    } else {
        let home = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No home dir"))?;
        home.join(".config")
    };

    Ok(base.join("gh-pr-tui").join("config.json"))
}

pub fn load(path: &Path) -> io::Result<AppConfig> {
    match fs::read_to_string(path) {
        Ok(s) => {
            let cfg: AppConfig = serde_json::from_str(&s)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Config parse error: {e}")))?;
            Ok(cfg)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(AppConfig::default()),
        Err(e) => Err(e),
    }
}

pub fn save(path: &Path, cfg: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(cfg)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Config serialize error: {e}")))?;
    fs::write(path, s)?;
    Ok(())
}