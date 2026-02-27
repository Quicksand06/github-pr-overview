use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum ReviewStatus {
    Approved,
    ChangesRequested,
    ReviewRequired,
    NotReviewed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PrOverviewRow {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub status: ReviewStatus,
    pub author: String,
    pub requested_reviewers: Vec<String>,
    pub latest_reviews: Vec<(String, String)>, // (login, state)
}

#[derive(Debug, Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQlError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RepoOpenPrsData {
    pub repository: Option<Repository>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
    #[serde(rename = "pullRequests")]
    pub pull_requests: PullRequestConnection,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestConnection {
    pub nodes: Option<Vec<PullRequestNode>>,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestNode {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub author: Option<UserLike>,
    #[serde(rename = "reviewDecision")]
    pub review_decision: Option<String>,
    #[serde(rename = "reviewRequests")]
    pub review_requests: ReviewRequestConnection,
    pub reviews: ReviewConnection,
}

#[derive(Debug, Deserialize)]
pub struct UserLike {
    pub login: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequestConnection {
    pub nodes: Option<Vec<ReviewRequestNode>>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequestNode {
    #[serde(rename = "requestedReviewer")]
    pub requested_reviewer: Option<RequestedReviewer>,
}

#[derive(Debug, Deserialize)]
pub struct RequestedReviewer {
    pub login: Option<String>, // User
    pub name: Option<String>,  // Team
    #[serde(rename = "__typename")]
    pub typename_: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewConnection {
    pub nodes: Option<Vec<ReviewNode>>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewNode {
    pub state: String, // APPROVED / CHANGES_REQUESTED / COMMENTED / DISMISSED ...
    pub author: Option<UserLike>,
}

impl ReviewStatus {
    pub fn from_review_decision(
        decision: Option<&str>,
        requested_reviewers_nonempty: bool,
    ) -> Self {
        match decision {
            Some("APPROVED") => ReviewStatus::Approved,
            Some("CHANGES_REQUESTED") => ReviewStatus::ChangesRequested,
            Some("REVIEW_REQUIRED") => ReviewStatus::ReviewRequired,
            Some(_) => ReviewStatus::Unknown,
            None => {
                // When reviewDecision is null, it may mean no review required by rules and none requested/submitted yet. :contentReference[oaicite:1]{index=1}
                if requested_reviewers_nonempty {
                    ReviewStatus::ReviewRequired
                } else {
                    ReviewStatus::NotReviewed
                }
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewStatus::Approved => "APPROVED",
            ReviewStatus::ChangesRequested => "CHANGES_REQUESTED",
            ReviewStatus::ReviewRequired => "REVIEW_REQUIRED",
            ReviewStatus::NotReviewed => "NOT_REVIEWED",
            ReviewStatus::Unknown => "UNKNOWN",
        }
    }
}
