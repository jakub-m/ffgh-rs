use crate::fzf::is_mute;
use crate::gh::PullRequest;
use crate::storage::{get_pr_state_flags, UserState, HAS_NEW_COMMENTS, IS_NEW, IS_UPDATED};
use std::io::Write;

pub fn print_compact_summary<W: Write>(
    writer: &mut W,
    prs: &[PullRequest],
    user_state: &UserState,
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

    //let mut parts = vec![format!("GH{}", total_count)];
    //if new_count > 0 {
    //    parts.push(format!("N{}", new_count));
    //}
    //if updated_count > 0 {
    //    parts.push(format!("U{}", updated_count));
    //}
    //if commented_count > 0 {
    //    parts.push(format!("C{}", commented_count));
    //}
    let mut parts = vec![format!("{}", total_count)];
    parts.push(format!("{}", new_count));
    parts.push(format!("{}", updated_count));
    parts.push(format!("{}", commented_count));

    write!(writer, "{}", parts.join(":"))?;
    Ok(())
}

