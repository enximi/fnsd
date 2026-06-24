use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use fns_config::AppConfig;
use fns_daemon::Daemon;
use fns_sync_engine::SyncEngine;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, writer::BoxMakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Parser)]
#[command(name = "fns-client-headless")]
#[command(version, about = "Headless FNS client")]
struct Cli {
    #[arg(short, long, global = true, default_value = "fns-headless.toml")]
    config: PathBuf,
    #[arg(long, global = true, default_value = "info", env = "FNS_HEADLESS_LOG")]
    log_level: String,
    #[arg(long, global = true, env = "FNS_HEADLESS_LOG_FILE")]
    log_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Sync {
        #[command(subcommand)]
        command: SyncCommand,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Check,
}

#[derive(Debug, Subcommand)]
enum SyncCommand {
    Once,
}

#[derive(Debug, Subcommand)]
enum DaemonCommand {
    Run,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let _log_guard = match init_logging(&cli.log_level, cli.log_file.clone()) {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Config {
            command: ConfigCommand::Check,
        } => {
            info!(config = %cli.config.display(), "checking config");
            AppConfig::load(&cli.config)?;
            println!("config ok: {}", cli.config.display());
        }
        Command::Sync {
            command: SyncCommand::Once,
        } => {
            info!(config = %cli.config.display(), "starting sync once");
            let config = AppConfig::load(&cli.config)?;
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
        }
        Command::Daemon {
            command: DaemonCommand::Run,
        } => {
            info!(config = %cli.config.display(), "starting daemon");
            let config = AppConfig::load(&cli.config)?;
            Daemon::new(config).run().await?;
        }
    }

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
