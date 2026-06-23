use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use fns_config::AppConfig;
use fns_sync_engine::SyncEngine;

#[derive(Debug, Parser)]
#[command(name = "fns-client-headless")]
#[command(version, about = "Headless FNS client")]
struct Cli {
    #[arg(short, long, global = true, default_value = "fns-headless.toml")]
    config: PathBuf,
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
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Check,
}

#[derive(Debug, Subcommand)]
enum SyncCommand {
    Once,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
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
            AppConfig::load(&cli.config)?;
            println!("config ok: {}", cli.config.display());
        }
        Command::Sync {
            command: SyncCommand::Once,
        } => {
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
    }

    Ok(())
}
