use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

use super::state::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(Line::from("GitHub PR Overview — Repo management"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("gh-pr-tui"));
    f.render_widget(header, root[0]);

    match app.mode {
        Mode::AddingRepo => draw_add_popup(f, app),
        Mode::ConfirmDelete | Mode::ErrorPopup => draw_message_popup(f, app),
        Mode::Normal => {}
    }
    // Body: repos table (top) + PR overview (bottom)
    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(root[1]);

    draw_repos_table(f, app, body[0]);
    draw_pr_overview_table(f, app, body[1]);

    // Footer
    // Footer / Legend
    let footer = Paragraph::new(Text::from(vec![
        Line::from("Navigation: ↑/↓ or j/k"),
        Line::from("a = add   d = delete   r = refresh"),
        Line::from("Enter = confirm   Esc = cancel   q = quit   Ctrl+C = quit"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title("Key bindings"));

    f.render_widget(footer, root[2]);
}

fn draw_repos_table(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = app
        .cfg
        .repos
        .iter()
        .enumerate()
        .map(|(i, r)| Row::new(vec![Cell::from((i + 1).to_string()), Cell::from(r.clone())]));

    let table = Table::new(rows, [Constraint::Length(4), Constraint::Min(10)])
        .header(Row::new(vec![
            Cell::from("#").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Repo").style(Style::default().add_modifier(Modifier::BOLD)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Monitored repos"),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(table, area, &mut app.table);
}

fn draw_pr_overview_table(f: &mut Frame, app: &mut App, area: Rect) {
    // Columns:
    // Repo | # | Status | Author | Requested reviewers | Latest reviews
    let rows = app.pr_rows.iter().map(|r| {
        let requested = if r.requested_reviewers.is_empty() {
            "-".to_string()
        } else {
            join_truncated(&r.requested_reviewers, 3, ", ")
        };

        // show last up to 3 reviews (from oldest->newest in API slice, we render newest-ish by taking from end)
        let latest = if r.latest_reviews.is_empty() {
            "-".to_string()
        } else {
            let mut items = Vec::new();
            for (user, state) in r.latest_reviews.iter().rev().take(3) {
                items.push(format!("{user}:{state}"));
            }
            items.reverse(); // keep them readable left->right
            items.join(" | ")
        };

        Row::new(vec![
            Cell::from(r.repo.clone()),
            Cell::from(r.number.to_string()),
            Cell::from(r.status.as_str()),
            Cell::from(r.author.clone()),
            Cell::from(requested),
            Cell::from(latest),
        ])
    });

    // Keep widths reasonable; the last two columns get most space
    let table = Table::new(
        rows,
        [
            Constraint::Length(22), // Repo (nameWithOwner)
            Constraint::Length(6),  // #
            Constraint::Length(18), // Status
            Constraint::Length(16), // Author
            Constraint::Min(18),    // Requested reviewers
            Constraint::Min(18),    // Latest reviews
        ],
    )
    .header(Row::new(vec![
        Cell::from("Repo").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("#").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Author").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Requested reviewers").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Latest reviews").style(Style::default().add_modifier(Modifier::BOLD)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Open PRs overview"),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(table, area, &mut app.pr_selected);
}

fn join_truncated(items: &[String], max: usize, sep: &str) -> String {
    if items.len() <= max {
        return items.join(sep);
    }
    let head = items
        .iter()
        .take(max)
        .cloned()
        .collect::<Vec<_>>()
        .join(sep);
    format!("{head} …(+{})", items.len() - max)
}

fn draw_add_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 30, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add repo (Enter=save, Esc=cancel)");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let hint = Paragraph::new("Accepted: OWNER/REPO, github.com/OWNER/REPO, full https URL")
        .wrap(Wrap { trim: true });

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Repo URL"));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .margin(1)
        .split(inner);

    f.render_widget(hint, chunks[0]);
    f.render_widget(input, chunks[1]);
}

fn draw_message_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.mode {
        Mode::ConfirmDelete => "Confirm",
        Mode::ErrorPopup => "Error",
        _ => "",
    };

    let p = Paragraph::new(app.message.as_str())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
