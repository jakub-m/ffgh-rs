use anyhow::Result;
use std::fs;

include!(concat!(env!("OUT_DIR"), "/ffgh_config_proto_types.rs"));

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
