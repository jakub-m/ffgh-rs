use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: String,
    #[serde(rename = "is_bot")]
    pub is_bot: bool,
    pub login: String,
    #[serde(rename = "type")]
    pub author_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    #[serde(rename = "nameWithOwner")]
    pub name_with_owner: String,
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