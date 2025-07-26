use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub queries: Vec<Query>,
    pub display_order: Vec<String>,
    pub attribution_order: Vec<String>,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub github_arg: String,
    pub query_name: String,
    pub short_name: String,
    #[serde(default)]
    pub mute: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            queries: vec![
                Query {
                    github_arg: "--assignee=@me".to_string(),
                    query_name: "Assignee".to_string(),
                    short_name: "a".to_string(),
                    mute: false,
                },
                Query {
                    github_arg: "--author=@me".to_string(),
                    query_name: "Author".to_string(),
                    short_name: "*".to_string(),
                    mute: false,
                },
                Query {
                    github_arg: "--mentions=@me".to_string(),
                    query_name: "Mentions".to_string(),
                    short_name: "m".to_string(),
                    mute: true,
                },
                Query {
                    github_arg: "--review-requested=@me".to_string(),
                    query_name: "ReviewRequested".to_string(),
                    short_name: "r".to_string(),
                    mute: false,
                },
            ],
            attribution_order: vec![
                "Assignee".to_string(),
                "Author".to_string(),
                "Mentions".to_string(),
                "ReviewRequested".to_string(),
            ],
            display_order: vec![
                "Mentions".to_string(),
                "ReviewRequested".to_string(),
                "Assignee".to_string(),
                "Author".to_string(),
            ],
            annotations: vec!["Approved".to_string()],
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_yaml() -> &'static str {
        r#"
queries:
  - github_arg: "--assignee=@me"
    query_name: "Assignee"
    short_name: "a"
  - github_arg: "--author=@me"
    query_name: "Author"
    short_name: "*"
  - github_arg: "--mentions=@me"
    query_name: "Mentions"
    short_name: "m"
    mute: true
  - github_arg: "--review-requested=@me"
    query_name: "ReviewRequested"
    short_name: "r"
attribution_order:
  - "Assignee"
  - "Author"
  - "Mentions"
  - "ReviewRequested"
display_order:
  - "Mentions"
  - "ReviewRequested"
  - "Assignee"
  - "Author"
annotations:
  - Approved
"#
    }
}