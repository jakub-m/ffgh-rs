use crate::config::Config;
use crate::gh::{Meta, PullRequest};
use crate::storage::Storage;
use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time;

const JSON_FIELDS: &str = "author,body,commentsCount,createdAt,id,number,repository,state,title,updatedAt,url";

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
        log::info!("Run gh search");
        
        let mut queried_prs: HashMap<String, Vec<PullRequest>> = HashMap::new();
        
        for query in &config.queries {
            let prs = get_prs(&query.github_arg, &query.query_name, query.mute).await?;
            for pr in prs {
                queried_prs.entry(pr.url.clone()).or_insert_with(Vec::new).push(pr);
            }
        }
        
        log::info!("Got {} PRs (with duplicates)", queried_prs.len());
        log::info!("Use attribution order: {:?}", config.attribution_order);
        
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
        
        self.storage.reset_pull_requests(unique_prs.clone())?;
        log::info!("Updated {} pull requests", unique_prs.len());
        
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

async fn get_prs(query: &str, meta_label: &str, mute: bool) -> Result<Vec<PullRequest>> {
    log::info!("Get PRs for: {}", query);
    
    let output = Command::new("gh")
        .args(&[
            "search", "prs",
            "--draft=false",
            "--state=open",
            query,
            "--json",
            JSON_FIELDS,
        ])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gh command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let mut prs: Vec<PullRequest> = serde_json::from_slice(&output.stdout)?;
    
    for pr in &mut prs {
        pr.meta = Meta {
            label: meta_label.to_string(),
            default_mute: mute,
        };
    }
    
    Ok(prs)
}