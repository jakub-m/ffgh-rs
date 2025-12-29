use crate::fzf::is_mute;
use crate::gh::PullRequest;
use crate::storage::{get_pr_state_flags, UserState, HAS_NEW_COMMENTS, IS_NEW, IS_UPDATED};
use std::io::Write;

const P_TOT: &str = "TOT";
const P_NEW: &str = "NEW";
const P_UPDATED: &str = "UPD";
const P_COMMENTED: &str = "COM";
pub const DEFAULT_FORMAT: &str = "t:TOT n:NEW u:UPD c:COM";

pub fn print_compact_summary<W: Write>(
    writer: &mut W,
    prs: &[PullRequest],
    user_state: &UserState,
    format: &str,
) -> Result<(), std::io::Error> {
    let mut new_count = 0;
    let mut updated_count = 0;
    let mut commented_count = 0;
    let mut total_count = 0;

    for pr in prs {
        if is_mute(user_state, pr) {
            continue;
        }

        total_count += 1;
        let pr_state = user_state.per_url.get(&pr.url).cloned().unwrap_or_default();
        let flags = get_pr_state_flags(pr, &pr_state);

        if flags & IS_NEW != 0 {
            new_count += 1;
        } else if flags & IS_UPDATED != 0 {
            updated_count += 1;
        } else if flags & HAS_NEW_COMMENTS != 0 {
            commented_count += 1;
        }
    }

    let mut s = format.to_string();
    s = s.replace(P_TOT, &format!("{total_count}")).to_string();
    s = s.replace(P_NEW, &format!("{new_count}")).to_string();
    s = s
        .replace(P_UPDATED, &format!("{updated_count}"))
        .to_string();
    s = s
        .replace(P_COMMENTED, &format!("{commented_count}"))
        .to_string();
    write!(writer, "{s}")?;
    Ok(())
}

// const DEFAULT_FORMAT: &str = "NEW||UPDATED||COMMENTED";
