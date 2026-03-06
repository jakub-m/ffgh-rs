use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Author {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "is_bot")]
    pub is_bot: bool,
    #[serde(default)]
    pub login: String,
    #[serde(default, rename = "type")]
    pub author_type: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub author: Author,
    pub body: String,
    #[serde(rename = "commentsCount")]
    pub comments_count: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    pub id: String,
    pub number: i32,
    pub repository: Repository,
    pub title: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub url: String,
    pub state: String,
    #[serde(rename = "reviewRequests", default)]
    pub review_requests: Vec<ReviewRequest>,
    #[serde(rename = "_meta", default)]
    pub meta: Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Meta {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub default_mute: bool,
}