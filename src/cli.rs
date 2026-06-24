use std::path::PathBuf;
use std::process::ExitCode;

use crate::config::AppConfig;
use crate::daemon::Daemon;
use crate::sync::engine::SyncEngine;
use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, writer::BoxMakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Parser)]
#[command(name = "fnsd")]
#[command(version, about = "FNS sync daemon")]
struct Cli {
    #[arg(short, long, global = true, default_value = "fnsd.toml")]
    config: PathBuf,
    #[arg(long, global = true, default_value = "info", env = "FNSD_LOG")]
    log_level: String,
    #[arg(long, global = true, env = "FNSD_LOG_FILE")]
    log_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Validate the configuration file")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    #[command(about = "Run one-shot synchronization")]
    Sync {
        #[command(subcommand)]
        command: SyncCommand,
    },
    #[command(about = "Run the long-lived sync daemon")]
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    #[command(about = "Load and validate the configuration")]
    Check,
}

#[derive(Debug, Subcommand)]
enum SyncCommand {
    #[command(about = "Run one synchronization pass and exit")]
    Once,
}

#[derive(Debug, Subcommand)]
enum DaemonCommand {
    #[command(about = "Run startup sync, watch local changes, and reconnect on failures")]
    Run,
}

pub async fn run() -> ExitCode {
    let cli = Cli::parse();
    let _log_guard = match init_logging(&cli.log_level, cli.log_file.clone()) {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match run_command(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Config {
            command: ConfigCommand::Check,
        } => check_config(cli.config),
        Command::Sync {
            command: SyncCommand::Once,
        } => sync_once(cli.config).await,
        Command::Daemon {
            command: DaemonCommand::Run,
        } => run_daemon(cli.config).await,
    }
}

fn check_config(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!(config = %path.display(), "checking config");
    AppConfig::load(&path)?;
    println!("config ok: {}", path.display());
    Ok(())
}

async fn sync_once(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!(config = %path.display(), "starting sync once");
    let config = AppConfig::load(&path)?;
    let summary = SyncEngine::new(config).sync_once().await?;
    println!(
        "sync complete: sent notes={}, files={}, folders={}, settings={}; events={}, remote writes={}, deletes={}, renames={}, mtime updates={}, acks={}",
        summary.notes_sent,
        summary.files_sent,
        summary.folders_sent,
        summary.settings_sent,
        summary.events.text_events,
        summary.events.remote_writes,
        summary.events.remote_deletes,
        summary.events.remote_renames,
        summary.events.remote_mtime_updates,
        summary.events.acks
    );
    Ok(())
}

async fn run_daemon(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!(config = %path.display(), "starting daemon");
    let config = AppConfig::load(&path)?;
    Daemon::new(config).run().await?;
    Ok(())
}

fn init_logging(
    log_level: &str,
    log_file: Option<PathBuf>,
) -> Result<Option<WorkerGuard>, Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_new(log_level)
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("static log filter is valid");

    let console = fmt::layer()
        .with_target(false)
        .compact()
        .with_writer(BoxMakeWriter::new(std::io::stderr))
        .boxed();

    let Some(log_file) = log_file else {
        Registry::default().with(filter).with(console).init();
        return Ok(None);
    };

    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;
    let (writer, guard) = tracing_appender::non_blocking(file);
    let file = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(writer)
        .boxed();

    Registry::default()
        .with(filter)
        .with(console)
        .with(file)
        .init();

    Ok(Some(guard))
}
