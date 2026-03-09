use crate::config::Config;
use crate::gh::{Author, Meta, PullRequest, Repository, Review, ReviewRequest};
use crate::storage::Storage;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time;

const GRAPHQL_QUERY: &str = r#"
query($q: String!) {
  search(query: $q, type: ISSUE, first: 100) {
    nodes {
      ... on PullRequest {
        author { login }
        body
        comments { totalCount }
        createdAt
        id
        number
        repository { name nameWithOwner }
        title
        updatedAt
        url
        state
        reviewRequests(first: 100) {
          nodes {
            requestedReviewer {
              ... on User { login }
              ... on Team { name slug }
            }
          }
        }
        latestReviews(first: 100) {
          nodes {
            author { login }
            state
          }
        }
      }
    }
  }
}
"#;

pub struct Synchronizer<S: Storage> {
    storage: S,
    interval: Duration,
}

impl<S: Storage> Synchronizer<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            interval: Duration::from_secs(60),
        }
    }

    pub async fn run_blocking(&self, config: &Config) -> Result<()> {
        loop {
            if let Err(e) = self.run_once(config).await {
                log::error!("Sync error: {}", e);
                return Err(e);
            }
            time::sleep(self.interval).await;
        }
    }

    pub async fn run_once(&self, config: &Config) -> Result<()> {
        log::debug!("Run gh search");

        let mut queried_prs: HashMap<String, Vec<PullRequest>> = HashMap::new();

        for query in &config.queries {
            log::debug!("Querying: {} ({})", query.query_name, query.github_arg);
            let prs = get_prs(&query.github_arg, &query.query_name, query.mute).await?;
            log::debug!("Found {} PRs for query '{}'", prs.len(), query.query_name);
            for pr in prs {
                queried_prs
                    .entry(pr.url.clone())
                    .or_insert_with(Vec::new)
                    .push(pr);
            }
        }

        log::debug!("Got {} PRs (with duplicates)", queried_prs.len());
        log::debug!("Use attribution order: {:?}", config.attribution_order);

        let mut attribution_priority: HashMap<String, usize> = HashMap::new();
        for (i, query_name) in config.attribution_order.iter().enumerate() {
            attribution_priority.insert(query_name.clone(), i);
        }

        let mut unique_prs = Vec::new();
        for prs in queried_prs.values() {
            let selected = if prs.len() == 1 {
                prs[0].clone()
            } else {
                select_pr_with_attribution_priority(prs, &attribution_priority)
            };
            unique_prs.push(selected);
        }

        log::debug!("Storing {} unique pull requests", unique_prs.len());
        self.storage.reset_pull_requests(unique_prs.clone())?;
        log::debug!("Successfully updated {} pull requests", unique_prs.len());

        Ok(())
    }
}

fn select_pr_with_attribution_priority(
    prs: &[PullRequest],
    attribution_priority: &HashMap<String, usize>,
) -> PullRequest {
    let mut selected = prs[0].clone();

    for pr in prs {
        let selected_priority = attribution_priority
            .get(&selected.meta.label)
            .copied()
            .unwrap_or(usize::MAX);
        let pr_priority = attribution_priority
            .get(&pr.meta.label)
            .copied()
            .unwrap_or(usize::MAX);

        if pr_priority < selected_priority {
            selected = pr.clone();
        }
    }

    selected
}

async fn get_prs(github_arg: &str, meta_label: &str, mute: bool) -> Result<Vec<PullRequest>> {
    let search_qualifier = github_arg.trim_start_matches("--").replacen('=', ":", 1);
    let search_query = format!("is:pr is:open draft:false {}", search_qualifier);

    log::debug!("Executing GraphQL search: {}", search_query);

    let output = Command::new("gh")
        .args([
            "api",
            "graphql",
            "-f",
            &format!("query={}", GRAPHQL_QUERY),
            "-f",
            &format!("q={}", search_query),
        ])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gh api graphql failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response: GqlResponse = serde_json::from_slice(&output.stdout)?;

    let prs = response
        .data
        .search
        .nodes
        .into_iter()
        .map(|node| to_pull_request(node, meta_label, mute))
        .collect();

    Ok(prs)
}

fn to_pull_request(gql: GqlPullRequest, meta_label: &str, mute: bool) -> PullRequest {
    PullRequest {
        author: Author {
            login: gql.author.map(|a| a.login).unwrap_or_default(),
            ..Default::default()
        },
        body: gql.body.unwrap_or_default(),
        comments_count: gql.comments.total_count,
        created_at: gql.created_at,
        id: gql.id,
        number: gql.number,
        repository: gql.repository,
        title: gql.title,
        updated_at: gql.updated_at,
        url: gql.url,
        state: gql.state,
        review_requests: gql
            .review_requests
            .nodes
            .into_iter()
            .filter_map(|n| n.requested_reviewer)
            .map(|r| ReviewRequest {
                login: r.login,
                name: r.name,
                slug: r.slug,
            })
            .collect(),
        latest_reviews: gql
            .latest_reviews
            .nodes
            .into_iter()
            .map(|r| Review {
                author_login: r.author.map(|a| a.login).unwrap_or_default(),
                state: r.state,
            })
            .collect(),
        meta: Meta {
            label: meta_label.to_string(),
            default_mute: mute,
        },
    }
}

#[derive(Deserialize)]
struct GqlResponse {
    data: GqlData,
}

#[derive(Deserialize)]
struct GqlData {
    search: GqlSearch,
}

#[derive(Deserialize)]
struct GqlSearch {
    nodes: Vec<GqlPullRequest>,
}

#[derive(Deserialize)]
struct GqlPullRequest {
    author: Option<GqlAuthor>,
    body: Option<String>,
    comments: GqlComments,
    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,
    id: String,
    number: i32,
    repository: Repository,
    title: String,
    #[serde(rename = "updatedAt")]
    updated_at: DateTime<Utc>,
    url: String,
    state: String,
    #[serde(rename = "reviewRequests")]
    review_requests: GqlReviewRequests,
    #[serde(rename = "latestReviews")]
    latest_reviews: GqlLatestReviews,
}

#[derive(Deserialize)]
struct GqlAuthor {
    login: String,
}

#[derive(Deserialize)]
struct GqlComments {
    #[serde(rename = "totalCount")]
    total_count: i32,
}

#[derive(Deserialize)]
struct GqlReviewRequests {
    nodes: Vec<GqlReviewRequestNode>,
}

#[derive(Deserialize)]
struct GqlReviewRequestNode {
    #[serde(rename = "requestedReviewer")]
    requested_reviewer: Option<GqlRequestedReviewer>,
}

#[derive(Deserialize)]
struct GqlRequestedReviewer {
    #[serde(default)]
    login: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    slug: String,
}

#[derive(Deserialize)]
struct GqlLatestReviews {
    nodes: Vec<GqlReviewNode>,
}

#[derive(Deserialize)]
struct GqlReviewNode {
    author: Option<GqlAuthor>,
    state: String,
}
