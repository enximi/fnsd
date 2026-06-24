use std::time::Duration;

use crate::config::AppConfig;
use crate::sync::session::{SyncSession, SyncSessionCommand};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::daemon::{
    DaemonError, Result,
    watch_task::{WatchReceiver, spawn_watch_task},
};

#[derive(Debug)]
pub struct Daemon {
    config: AppConfig,
    options: DaemonOptions,
}

impl Daemon {
    pub fn new(config: AppConfig) -> Self {
        let options = DaemonOptions::from_config(&config);
        Self { config, options }
    }

    pub async fn run(&self) -> Result<()> {
        info!("starting daemon");

        let mut watch = if self.options.watch_enabled {
            let scan_options = self.config.scan_options()?;
            Some(spawn_watch_task(
                self.config.vault.root.clone(),
                scan_options,
            ))
        } else {
            None
        };

        let mut retry_delay = self.options.retry_min;

        loop {
            let (command_tx, command_rx) = mpsc::channel(1024);
            let session = SyncSession::new(self.config.clone());

            match run_session_until_disconnect(session, command_rx, &mut watch, command_tx).await {
                Ok(()) => {
                    retry_delay = self.options.retry_min;
                }
                Err(err) => {
                    error!(%err, retry_seconds = retry_delay.as_secs(), "session stopped");
                    tokio::time::sleep(retry_delay).await;
                    retry_delay = next_retry_delay(retry_delay, self.options.retry_max);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaemonOptions {
    pub watch_enabled: bool,
    pub retry_min: Duration,
    pub retry_max: Duration,
}

impl DaemonOptions {
    pub fn from_config(config: &AppConfig) -> Self {
        let retry_min = Duration::from_secs(config.daemon.retry_min_seconds);
        let retry_max = Duration::from_secs(config.daemon.retry_max_seconds).max(retry_min);

        Self {
            watch_enabled: config.daemon.watch_enabled,
            retry_min,
            retry_max,
        }
    }
}

impl Default for DaemonOptions {
    fn default() -> Self {
        Self {
            watch_enabled: true,
            retry_min: Duration::from_secs(5),
            retry_max: Duration::from_secs(300),
        }
    }
}

async fn run_session_until_disconnect(
    session: SyncSession,
    command_rx: mpsc::Receiver<SyncSessionCommand>,
    watch: &mut Option<WatchReceiver>,
    command_tx: mpsc::Sender<SyncSessionCommand>,
) -> Result<()> {
    tokio::select! {
        result = session.run(command_rx) => {
            result?;
            Ok(())
        }
        result = forward_watch_events(watch, command_tx) => {
            result
        }
    }
}

async fn forward_watch_events(
    watch: &mut Option<WatchReceiver>,
    command_tx: mpsc::Sender<SyncSessionCommand>,
) -> Result<()> {
    let Some(receiver) = watch else {
        std::future::pending::<()>().await;
        unreachable!("pending future never resolves");
    };

    loop {
        let event = receiver
            .recv()
            .await
            .ok_or(DaemonError::WatchTaskStopped)??;
        if command_tx
            .send(SyncSessionCommand::VaultEvent(event))
            .await
            .is_err()
        {
            warn!("session command channel closed");
            return Ok(());
        }
    }
}

fn next_retry_delay(current: Duration, max: Duration) -> Duration {
    current.saturating_mul(2).min(max)
}
