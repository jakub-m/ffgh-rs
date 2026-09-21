use crate::fzf::is_mute;
use crate::gh::PullRequest;
use crate::storage::{get_pr_state_flags, UserState, HAS_NEW_COMMENTS, IS_NEW, IS_UPDATED};
use std::io::Write;

pub const FLAG_SEPARATOR: &str = ";";

/// Machine-readable listing: one PR per line, tab-separated fields, so the
/// output can be piped into scripts without parsing the fzf/xbar formatting.
pub fn print_pull_requests<W: Write>(
    writer: &mut W,
    prs: &[PullRequest],
    user_state: &UserState,
) -> Result<(), std::io::Error> {
    let mut prs: Vec<&PullRequest> = prs.iter().collect();
    prs.sort_by(|a, b| {
        a.repository
            .name
            .cmp(&b.repository.name)
            .then(a.number.cmp(&b.number))
    });

    for pr in prs {
        let flags = pr_flags(pr, user_state).join(FLAG_SEPARATOR);
        let title = pr.title.replace(FLAG_SEPARATOR, " ");
        writeln!(writer, "{url}\t{flags}\t{title}", url = pr.url)?;
    }

    Ok(())
}

fn pr_flags(pr: &PullRequest, user_state: &UserState) -> Vec<&'static str> {
    let pr_state = user_state.per_url.get(&pr.url).cloned().unwrap_or_default();
    let state_flags = get_pr_state_flags(pr, &pr_state);

    let mut flags = Vec::new();

    if state_flags & IS_NEW != 0 {
        flags.push("new");
    }
    if state_flags & IS_UPDATED != 0 {
        flags.push("updated");
    }
    if state_flags & HAS_NEW_COMMENTS != 0 {
        flags.push("commented");
    }
    if pr.latest_reviews.iter().any(|r| r.state == "APPROVED") {
        flags.push("approved");
    }
    if pr
        .latest_reviews
        .iter()
        .any(|r| r.state == "CHANGES_REQUESTED")
    {
        flags.push("changes-requested");
    }
    if is_mute(user_state, pr) {
        flags.push("mute");
    }

    flags
}
