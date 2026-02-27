use std::io;

mod app;
mod github;
mod tui;

fn main() -> Result<(), io::Error> {
    // Load .env (PAT stays there)
    let _ = dotenvy::dotenv();
    let _pat = std::env::var("GITHUB_TOKEN").ok();

    let cfg_path = app::config::default_config_path()?;
    let cfg = app::config::load(&cfg_path)?;

    let mut terminal = tui::setup_terminal()?;
    let mut app = app::App::new(cfg_path, cfg);

    let res = app::run(&mut terminal, &mut app);

    tui::restore_terminal(&mut terminal)?;
    res
}
