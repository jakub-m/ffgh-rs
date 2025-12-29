//! Implements the actions upon PRs. The actios are defined in the config.

use crate::config::{self, Match};
use crate::gh::PullRequest;

pub fn apply_actions(config: &config::Config, prs: &Vec<PullRequest>) -> Vec<PullRequest> {
    let mut final_prs: Vec<PullRequest> = Vec::new();

    for pr in prs {
        let mut current_pr = Some(pr.clone());
        for action in &config.actions {
            current_pr = match current_pr {
                Some(ref pr) => action.act(pr),
                None => break,
            };
        }
        if let Some(pr) = current_pr {
            final_prs.push(pr);
        }
    }

    final_prs
}

trait Act {
    /// Act upon a [PullRequest]. Modifies the pull request, or removes it altogether.
    fn act(&self, pr: &PullRequest) -> Option<PullRequest>;
}

impl Act for config::Action {
    fn act(&self, pr: &PullRequest) -> Option<PullRequest> {
        let mut pr = pr.clone();
        if self.matches.is_empty() {
            return Some(pr);
        }
        if !self.matches.iter().any(|m| is_match(m, &pr)) {
            return Some(pr);
        }
        if self.mute {
            pr.meta.default_mute = true
        }
        Some(pr)
    }
}

/// Check if ALL of the clauses in [Match] match the pr.
fn is_match(m: &Match, pr: &PullRequest) -> bool {
    if !m.title.is_empty() {
        if !pr.title.contains(&m.title) {
            return false;
        }
    }
    true
}
