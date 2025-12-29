use anyhow::Result;
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use colored::control;
use ffgh::actions;
use ffgh::{
    config::Config, fzf, storage::FileStorage, storage::Storage, sync::Synchronizer, util, xbar,
};
use std::env;
use std::fs;
use std::io;
use std::path::Path;

const OUT_OF_SYNC_PERIOD_MINUTES: i64 = 5;

#[derive(Parser)]
#[command(name = "ffgh")]
#[command(version)]
#[command(about = "Utility to synchronize and display state of GitHub PRs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, help = "Verbose output")]
    verbose: bool,

    #[arg(short = 'd', long, help = "Directory where to store the state")]
    state_path: Option<String>,

    #[arg(short = 'c', long, help = "Config file path")]
    config_path: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "sync")]
    Sync {
        #[arg(long, help = "Run once instead of continuously")]
        once: bool,
    },
    #[command(name = "fzf")]
    Fzf,
    #[command(name = "show-compact-summary")]
    ShowCompactSummary,
    #[command(name = "show-pr")]
    ShowPr { url: String },
    #[command(name = "mark-open")]
    MarkOpen {
        url: String,
        #[arg(short = 'e', help = "Exit with error if already marked")]
        exit_error_if_marked: bool,
    },
    #[command(name = "mark-mute")]
    MarkMute { url: String },
    #[command(name = "add-note")]
    AddNote { url: String, note_file: String },
    #[command(name = "cycle-view-mode")]
    CycleViewMode,
    #[command(name = "cycle-note")]
    CycleNote { url: String },
    #[command(name = "config-check")]
    ConfigCheck,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Force colors to be enabled, similar to Go version's color.NoColor = false
    control::set_override(true);

    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    let default_state_dir = get_default_state_dir()?;
    let state_path = cli.state_path.unwrap_or(default_state_dir.clone());

    if state_path == default_state_dir {
        fs::create_dir_all(&state_path)?;
    }

    let default_config_path = Path::new(&state_path).join("config.yaml");
    let config_path = cli
        .config_path
        .unwrap_or_else(|| default_config_path.to_string_lossy().to_string());

    let config = if Path::new(&config_path).exists() {
        log::debug!("Reading config from {}", config_path);
        Config::from_file(&config_path).unwrap_or_else(|e| {
            log::warn!("Failed to read config, using default: {}", e);
            Config::default()
        })
    } else {
        log::debug!("Config file {} does not exist, using default", config_path);
        Config::default()
    };

    let mut storage = FileStorage::new();
    storage.prs_state_path = Path::new(&state_path)
        .join(&storage.prs_state_path)
        .to_string_lossy()
        .to_string();
    storage.user_state_path = Path::new(&state_path)
        .join(&storage.user_state_path)
        .to_string_lossy()
        .to_string();
    log::debug!("prs_state_path {:?}", storage.prs_state_path);
    log::debug!("user_state_path {:?}", storage.user_state_path);

    match cli.command {
        Commands::Sync { once } => {
            let synchronizer = Synchronizer::new(storage);
            if once {
                synchronizer.run_once(&config).await?;
            } else {
                synchronizer.run_blocking(&config).await?;
            }
        }
        Commands::Fzf => {
            let terminal_width = env::var("TERMINAL_WIDTH")
                .unwrap_or_else(|_| "120".to_string())
                .parse::<usize>()
                .unwrap_or(120);

            let prs = storage.get_pull_requests()?;
            let user_state = storage.get_user_state()?;

            let sync_str = if let Some(sync_time) = storage.get_sync_time() {
                let duration = Utc::now() - sync_time;
                format!("X synced {} ago", format_duration(duration))
            } else {
                "X not synced".to_string()
            };

            println!("{} | {}", sync_str, user_state.settings.view_mode);

            let prs = prs;
            fzf::print_pull_requests(
                &mut io::stdout(),
                terminal_width,
                &prs,
                &user_state,
                &config,
            )?;
        }
        Commands::ShowCompactSummary => {
            let prs = storage.get_pull_requests()?;
            let prs = actions::apply_actions(&config, &prs);
            let user_state = storage.get_user_state()?;

            let out_of_sync_time = Utc::now() - Duration::minutes(OUT_OF_SYNC_PERIOD_MINUTES);
            let compact_format = match &config.compact_format {
                s if s.is_empty() => xbar::DEFAULT_FORMAT,
                s => s,
            };
            if let Some(sync_time) = storage.get_sync_time() {
                if sync_time < out_of_sync_time {
                    print!("GH err!");
                } else {
                    xbar::print_compact_summary(
                        &mut io::stdout(),
                        &prs,
                        &user_state,
                        compact_format,
                    )?;
                }
            } else {
                print!("GH err!");
            }
        }
        Commands::ShowPr { url } => {
            let prs = storage.get_pull_requests()?;
            let user_state = storage.get_user_state()?;
            fzf::print_show_pull_request(&mut io::stdout(), &url, &prs, &user_state)?;
        }
        Commands::MarkOpen {
            url,
            exit_error_if_marked,
        } => {
            let marked = storage.mark_url_as_opened(&url)?;
            if !marked && exit_error_if_marked {
                return Err(anyhow::anyhow!(
                    "URL already marked as opened, doing nothing: {}",
                    url
                ));
            }
        }
        Commands::MarkMute { url } => {
            storage.mark_url_as_muted(&url)?;
        }
        Commands::AddNote { url, note_file } => {
            let note = fs::read_to_string(note_file)?.trim().to_string();
            storage.add_note(&url, &note)?;
        }
        Commands::CycleViewMode => {
            let mut user_state = storage.get_user_state()?;
            let old_mode = user_state.settings.view_mode.clone();
            user_state.settings.view_mode = fzf::cycle_view_mode(&old_mode);
            log::debug!(
                "Changed view mode from {} to {}",
                old_mode,
                user_state.settings.view_mode
            );
            storage.write_user_state(&user_state)?;
        }
        Commands::CycleNote { url } => {
            if config.annotations.is_empty() {
                return Err(anyhow::anyhow!("No annotations set in config"));
            }

            let user_state = storage.get_user_state()?;
            let current_note = user_state
                .per_url
                .get(&url)
                .map(|state| state.note.as_str())
                .unwrap_or("");

            let mut annotations = config.annotations.clone();
            annotations.push(String::new()); // Add empty note at the end

            let new_note = util::cycle(current_note, &annotations);
            log::debug!("Cycling note from '{}' to '{}'", current_note, new_note);
            storage.add_note(&url, &new_note)?;
        }
        Commands::ConfigCheck => match serde_yaml::to_string(&config) {
            Ok(s) => {
                println!("{s}")
            }
            Err(e) => {
                println!("FAILED! {e}")
            }
        },
    }

    Ok(())
}

fn get_default_state_dir() -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".ffgh").to_string_lossy().to_string())
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        format!("{}m", total_seconds / 60)
    } else if total_seconds < 86400 {
        format!("{}h", total_seconds / 3600)
    } else {
        format!("{}d", total_seconds / 86400)
    }
}
