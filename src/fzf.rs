use crate::config::Config;
use crate::gh::PullRequest;
use crate::storage::{get_pr_state_flags, UserState, HAS_NEW_COMMENTS, IS_NEW, IS_UPDATED};
use crate::util;
use chrono::{Duration, Utc};
use colored::*;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::io::Write;

pub const VIEW_MODE_REGULAR: &str = "regular";
pub const VIEW_MODE_MUTE_TOP: &str = "mute-top";
pub const VIEW_MODE_HIDE_MUTE: &str = "hide-mute";

const NBSP: &str = "\u{00A0}";

pub fn cycle_view_mode(mode: &str) -> String {
    let view_modes = vec![
        VIEW_MODE_REGULAR.to_string(),
        VIEW_MODE_MUTE_TOP.to_string(),
        VIEW_MODE_HIDE_MUTE.to_string(),
    ];
    util::cycle(mode, &view_modes)
}

pub fn is_mute(user_state: &UserState, pr: &PullRequest) -> bool {
    if let Some(state) = user_state.per_url.get(&pr.url) {
        state.is_mute
    } else {
        pr.meta.default_mute
    }
}

pub fn print_pull_requests<W: Write>(
    writer: &mut W,
    terminal_width: usize,
    prs: &mut Vec<PullRequest>,
    user_state: &UserState,
    config: &Config,
) -> Result<(), std::io::Error> {
    log::info!("Use terminal width of {}", terminal_width);

    let mut display_priority: HashMap<String, usize> = HashMap::new();
    for (i, query_name) in config.display_order.iter().enumerate() {
        display_priority.insert(query_name.clone(), i);
    }

    prs.sort_by(|a, b| a.number.cmp(&b.number));
    prs.sort_by(|a, b| a.repository.name.cmp(&b.repository.name));
    prs.sort_by(|a, b| {
        let a_priority = display_priority.get(&a.meta.label).copied().unwrap_or(usize::MAX);
        let b_priority = display_priority.get(&b.meta.label).copied().unwrap_or(usize::MAX);
        a_priority.cmp(&b_priority)
    });

    let filtered_prs = match user_state.settings.view_mode.as_str() {
        VIEW_MODE_MUTE_TOP => {
            let mut not_muted: Vec<_> = prs.iter().filter(|pr| !is_mute(user_state, pr)).cloned().collect();
            let muted: Vec<_> = prs.iter().filter(|pr| is_mute(user_state, pr)).cloned().collect();
            not_muted.extend(muted);
            not_muted
        }
        VIEW_MODE_HIDE_MUTE => {
            prs.iter().filter(|pr| !is_mute(user_state, pr)).cloned().collect()
        }
        _ => prs.clone(),
    };

    let repo_name_max_len = filtered_prs
        .iter()
        .map(|pr| pr.repository.name.len())
        .max()
        .unwrap_or(0);

    for pr in &filtered_prs {
        let pr_state = user_state.per_url.get(&pr.url).cloned().unwrap_or_default();
        let flags = get_pr_state_flags(pr, &pr_state);
        let mute = is_mute(user_state, pr);

        let mut flag_string = String::new();
        
        if flags & IS_NEW != 0 {
            if mute {
                flag_string.push_str("N");
            } else {
                flag_string.push_str(&"N".green().to_string());
            }
        } else {
            flag_string.push_str(NBSP);
        }
        
        if flags & IS_UPDATED != 0 {
            if mute {
                flag_string.push_str("U");
            } else {
                flag_string.push_str(&"U".bright_white().to_string());
            }
        } else {
            flag_string.push_str(NBSP);
        }
        
        if flags & HAS_NEW_COMMENTS != 0 {
            if mute {
                flag_string.push_str("C");
            } else {
                flag_string.push_str(&"C".bright_yellow().to_string());
            }
        } else {
            flag_string.push_str(NBSP);
        }

        let note = if !pr_state.note.is_empty() {
            if mute {
                format!(" [{}]", pr_state.note)
            } else {
                format!(" [{}]", pr_state.note.cyan())
            }
        } else {
            String::new()
        };

        let short_label = config
            .queries
            .iter()
            .find(|q| q.query_name == pr.meta.label)
            .map(|q| q.short_name.as_str())
            .unwrap_or(" ");

        let left_parts = vec![
            flag_string,
            format!("{:<width$}", pr.repository.name, width = repo_name_max_len),
            short_label.to_string(),
            format!("#{:<5}", pr.number),
            pr.title.clone(),
        ];

        let line_left = left_parts.join(" ");
        let line_right = note;
        let line = format!("{}\t{}", pr.url, join_strings_cap_width(terminal_width, &line_left, &line_right));

        let final_line = if mute {
            line.bright_black().to_string()
        } else {
            line
        };

        writeln!(writer, "{}", final_line)?;
    }

    Ok(())
}

pub fn print_show_pull_request<W: Write>(
    writer: &mut W,
    pr_url: &str,
    prs: &[PullRequest],
    user_state: &UserState,
) -> Result<(), std::io::Error> {
    let pr = prs.iter().find(|p| p.url == pr_url);
    if let Some(pr) = pr {
        let pr_state = user_state.per_url.get(&pr.url).cloned().unwrap_or_default();
        let note = if !pr_state.note.is_empty() {
            format!("[{}]", pr_state.note.yellow())
        } else {
            String::new()
        };

        let flags = get_pr_state_flags(pr, &pr_state);
        let mut flag_string = String::new();

        if flags & IS_NEW != 0 {
            flag_string.push_str(&format!("{} ", "NEW".green()));
        }
        if flags & IS_UPDATED != 0 {
            flag_string.push_str(&format!("{} ", "UPDATED".bright_white()));
        }
        if flags & HAS_NEW_COMMENTS != 0 {
            flag_string.push_str(&"COMMENTS".bright_yellow().to_string());
        }

        let now = Utc::now();
        let details = vec![
            pr.repository.name_with_owner.bright_red().to_string(),
            format!("(#{}) {}", pr.number, pr.title).cyan().to_string(),
            String::new(),
            flag_string,
            format!("{} ({})", pr.author.login, pr.meta.label).yellow().to_string(),
            format!(
                "Created {}, updated {} ago",
                PrettyDuration::from_duration(now - pr.created_at),
                PrettyDuration::from_duration(now - pr.updated_at)
            ).yellow().to_string(),
            format!("{} comment(s)", pr.comments_count).yellow().to_string(),
            note,
            String::new(),
            pr.body.clone(),
        ];

        for detail in details {
            writeln!(writer, "{}", detail)?;
        }
    }

    Ok(())
}

fn join_strings_cap_width(width: usize, left: &str, right: &str) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    
    if left_len + right_len <= width {
        return format!("{}{}", left, right);
    }
    
    let mut result = String::with_capacity(width);
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    
    let available_for_left = width.saturating_sub(right_len);
    
    for (i, &ch) in left_chars.iter().enumerate() {
        if i >= available_for_left {
            break;
        }
        if i == available_for_left - 1 && available_for_left < left_len {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    
    if right_len > 0 && result.len() + right_len <= width {
        let padding = width - result.len() - right_len;
        for _ in 0..padding {
            result.push(' ');
        }
        for &ch in &right_chars {
            result.push(ch);
        }
    }
    
    result
}

#[derive(Debug, Clone, Copy)]
pub struct PrettyDuration {
    duration: Duration,
}

impl PrettyDuration {
    pub fn from_duration(duration: Duration) -> Self {
        Self { duration }
    }
}

impl Display for PrettyDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_seconds = self.duration.num_seconds();
        let days = total_seconds / 86400;
        let remaining = total_seconds % 86400;
        let hours = remaining / 3600;
        let remaining = remaining % 3600;
        let minutes = remaining / 60;
        let seconds = remaining % 60;

        if days > 0 {
            if hours > 0 || minutes > 0 || seconds > 0 {
                write!(f, "{}d", days)?;
                if hours > 0 {
                    write!(f, "{}h", hours)?;
                }
                if minutes > 0 {
                    write!(f, "{}m", minutes)?;
                }
                if seconds > 0 {
                    write!(f, "{}s", seconds)?;
                }
            } else {
                write!(f, "{}d", days)?;
            }
        } else if hours > 0 {
            write!(f, "{}h", hours)?;
            if minutes > 0 {
                write!(f, "{}m", minutes)?;
            }
            if seconds > 0 && minutes == 0 {
                write!(f, "{}s", seconds)?;
            }
        } else if minutes > 0 {
            write!(f, "{}m", minutes)?;
            if seconds > 0 {
                write!(f, "{}s", seconds)?;
            }
        } else {
            write!(f, "{}s", seconds)?;
        }

        Ok(())
    }
}