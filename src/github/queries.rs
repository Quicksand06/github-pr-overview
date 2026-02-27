pub const REPO_OPEN_PRS_QUERY: &str = r#"
query RepoOpenPRs($owner: String!, $name: String!, $first: Int!) {
  repository(owner: $owner, name: $name) {
    nameWithOwner
    pullRequests(states: OPEN, first: $first, orderBy: {field: UPDATED_AT, direction: DESC}) {
      nodes {
        number
        title
        url
        author {
          __typename
          ... on User { login }
        }
        reviewDecision
        reviewRequests(first: 20) {
          nodes {
            requestedReviewer {
              __typename
              ... on User { login }
              ... on Team { name }
            }
          }
        }
        reviews(last: 20) {
          nodes {
            state
            author {
              __typename
              ... on User { login }
            }
          }
        }
      }
    }
  }
}
"#;