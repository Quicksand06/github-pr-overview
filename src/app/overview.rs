use crate::github::{
    GitHubClient, PrOverviewRow, ReviewStatus,
    queries::REPO_OPEN_PRS_QUERY,
    types::{GraphQlResponse, RepoOpenPrsData},
};

pub fn fetch_repo_open_prs(
    gh: &GitHubClient,
    owner: &str,
    name: &str,
) -> Result<Vec<PrOverviewRow>, String> {
    let vars = serde_json::json!({
        "owner": owner,
        "name": name,
        "first": 50
    });

    let resp: GraphQlResponse<RepoOpenPrsData> = gh.graphql(REPO_OPEN_PRS_QUERY, vars)?;

    if let Some(errors) = resp.errors {
        let joined = errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(joined);
    }

    let repo = resp
        .data
        .and_then(|d| d.repository)
        .ok_or_else(|| "Repository not found or inaccessible".to_string())?;

    let nodes = repo.pull_requests.nodes.unwrap_or_default();

    let mut rows = Vec::with_capacity(nodes.len());

    for pr in nodes {
        let author = pr
            .author
            .and_then(|a| a.login)
            .unwrap_or_else(|| "unknown".into());

        let requested_reviewers = pr
            .review_requests
            .nodes
            .unwrap_or_default()
            .into_iter()
            .filter_map(|n| n.requested_reviewer)
            .map(|r| match r.typename_.as_str() {
                "User" => r.login.unwrap_or_else(|| "unknown".into()),
                "Team" => format!("team:{}", r.name.unwrap_or_else(|| "unknown".into())),
                _ => "unknown".into(),
            })
            .collect::<Vec<_>>();

        let requested_nonempty = !requested_reviewers.is_empty();

        let status =
            ReviewStatus::from_review_decision(pr.review_decision.as_deref(), requested_nonempty);

        // Summarize latest reviews (keep last 20 from API result)
        let latest_reviews = pr
            .reviews
            .nodes
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| {
                let who = r
                    .author
                    .and_then(|a| a.login)
                    .unwrap_or_else(|| "unknown".into());
                Some((who, r.state))
            })
            .collect::<Vec<_>>();

        rows.push(PrOverviewRow {
            repo: repo.name_with_owner.clone(),
            number: pr.number,
            title: pr.title,
            url: pr.url,
            status,
            author,
            requested_reviewers,
            latest_reviews,
        });
    }

    Ok(rows)
}
