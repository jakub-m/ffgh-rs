use crate::gh::PullRequest;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub trait Storage {
    fn reset_pull_requests(&self, prs: Vec<PullRequest>) -> Result<()>;
    fn get_pull_requests(&self) -> Result<Vec<PullRequest>>;
    fn mark_url_as_opened(&self, url: &str) -> Result<bool>;
    fn mark_url_as_muted(&self, url: &str) -> Result<()>;
    fn get_user_state(&self) -> Result<UserState>;
    fn write_user_state(&self, state: &UserState) -> Result<()>;
    fn get_sync_time(&self) -> Option<DateTime<Utc>>;
    fn add_note(&self, url: &str, note: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct FileStorage {
    pub prs_state_path: String,
    pub user_state_path: String,
}

impl FileStorage {
    pub fn new() -> Self {
        Self {
            prs_state_path: "gh_daemon_state.json".to_string(),
            user_state_path: "gh_user_state.json".to_string(),
        }
    }

    fn get_pr_for_url(&self, url: &str) -> Result<PullRequest> {
        let prs = self.get_pull_requests()?;
        prs.into_iter()
            .find(|pr| pr.url == url)
            .ok_or_else(|| anyhow::anyhow!("No PR found with URL: {}", url))
    }

    fn write_at_once(&self, target: &str, data: &[u8]) -> Result<()> {
        let temp_path = format!("{}.temp", target);
        
        // Remove existing temp file if it exists
        let _ = fs::remove_file(&temp_path);
        
        // Write to temp file
        fs::write(&temp_path, data)?;
        
        // Remove target file if it exists
        let _ = fs::remove_file(target);
        
        // Rename temp to target
        fs::rename(&temp_path, target)?;
        
        Ok(())
    }
}

impl Storage for FileStorage {
    fn reset_pull_requests(&self, prs: Vec<PullRequest>) -> Result<()> {
        let json = serde_json::to_string_pretty(&prs)?;
        self.write_at_once(&self.prs_state_path, json.as_bytes())
    }

    fn get_pull_requests(&self) -> Result<Vec<PullRequest>> {
        let content = fs::read_to_string(&self.prs_state_path)?;
        let prs: Vec<PullRequest> = serde_json::from_str(&content)?;
        Ok(prs)
    }

    fn mark_url_as_opened(&self, url: &str) -> Result<bool> {
        log::info!("Mark opened {}", url);
        let pr = self.get_pr_for_url(url)?;
        let mut user_state = self.get_user_state()?;
        
        let mut pr_state = user_state.per_url.get(url).cloned().unwrap_or_default();
        
        if pr_state.opened_at.is_some() 
            && pr_state.opened_at == Some(pr.updated_at)
            && pr_state.last_comment_count == pr.comments_count {
            log::info!("PR state up to date, not marking it as opened");
            return Ok(false);
        }
        
        log::info!("PR state changed so it's marked as opened");
        pr_state.opened_at = Some(pr.updated_at);
        pr_state.last_comment_count = pr.comments_count;
        user_state.per_url.insert(url.to_string(), pr_state);
        
        self.write_user_state(&user_state)?;
        Ok(true)
    }

    fn mark_url_as_muted(&self, url: &str) -> Result<()> {
        log::info!("Mark muted {}", url);
        let mut user_state = self.get_user_state()?;
        let mut pr_state = user_state.per_url.get(url).cloned().unwrap_or_default();
        
        pr_state.is_mute = !pr_state.is_mute;
        log::info!("Change mute state to {} {}", pr_state.is_mute, url);
        
        user_state.per_url.insert(url.to_string(), pr_state);
        self.write_user_state(&user_state)
    }

    fn get_user_state(&self) -> Result<UserState> {
        if !Path::new(&self.user_state_path).exists() {
            return Ok(UserState::default());
        }
        
        let content = fs::read_to_string(&self.user_state_path)?;
        let state: UserState = serde_json::from_str(&content)?;
        Ok(state)
    }

    fn write_user_state(&self, state: &UserState) -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        self.write_at_once(&self.user_state_path, json.as_bytes())
    }

    fn get_sync_time(&self) -> Option<DateTime<Utc>> {
        fs::metadata(&self.prs_state_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(|time| DateTime::from(time))
    }

    fn add_note(&self, url: &str, note: &str) -> Result<()> {
        log::info!("Add note to URL {}: {}", url, note);
        let mut user_state = self.get_user_state()?;
        let mut pr_state = user_state.per_url.get(url).cloned().unwrap_or_default();
        
        pr_state.note = note.to_string();
        user_state.per_url.insert(url.to_string(), pr_state);
        
        self.write_user_state(&user_state)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    #[serde(rename = "PerUrl", default)]
    pub per_url: HashMap<String, PrState>,
    #[serde(rename = "Settings", default)]
    pub settings: UserSettings,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            per_url: HashMap::new(),
            settings: UserSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(rename = "ViewMode", default = "default_view_mode")]
    pub view_mode: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            view_mode: default_view_mode(),
        }
    }
}

fn default_view_mode() -> String {
    "show-all".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrState {
    #[serde(rename = "OpenedAt")]
    pub opened_at: Option<DateTime<Utc>>,
    #[serde(rename = "LastCommentCount", default)]
    pub last_comment_count: i32,
    #[serde(rename = "Note", default)]
    pub note: String,
    #[serde(rename = "IsMute", default)]
    pub is_mute: bool,
}

pub const HAS_NEW_COMMENTS: u8 = 1 << 0;
pub const IS_UPDATED: u8 = 1 << 1;
pub const IS_NEW: u8 = 1 << 2;

pub fn get_pr_state_flags(pr: &PullRequest, pr_state: &PrState) -> u8 {
    let mut flags = 0;
    
    if pr.comments_count > pr_state.last_comment_count {
        flags |= HAS_NEW_COMMENTS;
    }
    
    if pr_state.opened_at.is_none() {
        flags |= IS_NEW;
    } else if let Some(opened_at) = pr_state.opened_at {
        if pr.updated_at > opened_at {
            flags |= IS_UPDATED;
        }
    }
    
    flags
}