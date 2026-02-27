pub fn normalize_repo_url(input: &str) -> Result<String, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Empty repo URL".into());
    }

    // Accept:
    // - https://github.com/OWNER/REPO
    // - github.com/OWNER/REPO
    // - OWNER/REPO
    // - git@github.com:OWNER/REPO(.git)
    let mut work = s.to_string();

    if work.starts_with("git@github.com:") {
        work = work.trim_start_matches("git@github.com:").to_string();
    } else {
        work = work
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.github.com/")
            .trim_start_matches("github.com/")
            .trim_start_matches("github.com")
            .trim_start_matches('/')
            .to_string();
    }

    work = work.trim_end_matches('/').to_string();
    work = work.trim_end_matches(".git").to_string();

    let mut parts = work.split('/').filter(|p| !p.is_empty());
    let owner = parts.next().ok_or("Missing owner")?;
    let repo = parts.next().ok_or("Missing repo")?;

    if parts.next().is_some() {
        return Err("Repo must be OWNER/REPO (no extra path segments)".into());
    }

    if !is_simple_slug(owner) || !is_simple_slug(repo) {
        return Err("Owner/repo contains invalid characters".into());
    }

    Ok(format!("https://github.com/{owner}/{repo}"))
}

fn is_simple_slug(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

pub fn owner_and_name(normalized_repo_url: &str) -> Result<(String, String), String> {
    // expects https://github.com/OWNER/REPO
    let s = normalized_repo_url.trim().trim_end_matches('/');
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() < 2 {
        return Err("Invalid repo URL".into());
    }
    let owner = parts.get(parts.len() - 2).ok_or("Missing owner")?.to_string();
    let name = parts.get(parts.len() - 1).ok_or("Missing repo")?.to_string();
    Ok((owner, name))
}