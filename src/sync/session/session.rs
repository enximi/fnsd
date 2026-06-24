use std::time::Duration;

use crate::config::AppConfig;
use crate::core::VaultName;
use crate::protocol::Action;
use crate::store::LocalStore;
use crate::sync::apply::{EventOutcome, apply_text_event};
use crate::sync::engine::{SyncEngine, TransferOptions};
use crate::sync::transfer::build_file_get_request;
use crate::vault::fs::VaultFs;
use crate::vault::watch::VaultWatchEvent;
use crate::ws::{ClientDescriptor, WebSocketClient, WsEvent};
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep_until};
use tracing::{debug, info};

use crate::sync::session::{
    Result, SyncSessionError,
    echo::RemoteEchoes,
    file_transfer::{ActiveDownloads, send_file_upload},
    local_change::send_local_changes,
    pending_watch::PendingWatchEvents,
};

#[derive(Debug)]
pub struct SyncSession {
    config: AppConfig,
    options: SyncSessionOptions,
}

impl SyncSession {
    pub fn new(config: AppConfig) -> Self {
        let options = SyncSessionOptions::from_config(&config);
        Self { config, options }
    }

    pub async fn run(&self, mut commands: mpsc::Receiver<SyncSessionCommand>) -> Result<()> {
        let vault_name = self.config.vault_name()?;
        let vault = VaultFs::new(&self.config.vault.root)?;
        let mut store = LocalStore::open(&self.config.store.path)?;
        let mut ws = self.connect(&vault, &mut store).await?;
        self.startup_sync(&mut ws, &vault, &mut store).await?;
        drain_startup_watch_events(&mut commands);
        let mut pending_change = None::<Instant>;
        let mut pending_events = PendingWatchEvents::default();
        let mut remote_echoes = RemoteEchoes::default();
        let mut downloads = ActiveDownloads::default();

        loop {
            match pending_change {
                Some(deadline) => {
                    tokio::select! {
                            _ = sleep_until(deadline) => {
                                pending_change = None;
                                self.handle_pending_changes(&mut ws, &vault_name, &vault, &mut store, &mut pending_events).await?;
                            }
                        command = commands.recv() => {
                            handle_command(command, &mut pending_change, &mut pending_events, &mut remote_echoes, self.options.debounce)?;
                        }
                        event = ws.next_event() => {
                            self.handle_ws_event(event?, &mut ws, &vault_name, &vault, &mut store, &mut downloads, &mut remote_echoes).await?;
                        }
                    }
                }
                None => {
                    tokio::select! {
                        command = commands.recv() => {
                            handle_command(command, &mut pending_change, &mut pending_events, &mut remote_echoes, self.options.debounce)?;
                        }
                        event = ws.next_event() => {
                            self.handle_ws_event(event?, &mut ws, &vault_name, &vault, &mut store, &mut downloads, &mut remote_echoes).await?;
                        }
                    }
                }
            }
        }
    }

    async fn startup_sync(
        &self,
        ws: &mut WebSocketClient,
        vault: &VaultFs,
        store: &mut LocalStore,
    ) -> Result<()> {
        let summary = SyncEngine::new(self.config.clone())
            .sync_authenticated(ws, vault, store)
            .await?;
        info!(
            notes = summary.notes_sent,
            files = summary.files_sent,
            folders = summary.folders_sent,
            settings = summary.settings_sent,
            events = summary.events.text_events,
            acks = summary.events.acks,
            "startup sync completed"
        );
        Ok(())
    }

    async fn connect(&self, vault: &VaultFs, store: &mut LocalStore) -> Result<WebSocketClient> {
        let ws_url = self
            .config
            .server
            .ws_url_with_protocol(self.config.client.protobuf)?;
        info!(server = %ws_url, "connecting long-lived websocket session");
        let mut ws = WebSocketClient::connect(&ws_url).await?;
        ws.authorize(self.config.server.api_token.clone()).await?;
        wait_for_authorization(&mut ws, vault, store).await?;
        send_client_info(&self.config, &mut ws, vault, store).await?;
        Ok(ws)
    }

    async fn handle_pending_changes(
        &self,
        ws: &mut WebSocketClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        pending_events: &mut PendingWatchEvents,
    ) -> Result<()> {
        send_local_changes(
            ws,
            vault_name,
            vault,
            store,
            &self.config,
            pending_events.take_all(),
        )
        .await?;
        Ok(())
    }

    async fn handle_ws_event(
        &self,
        event: WsEvent,
        ws: &mut WebSocketClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        downloads: &mut ActiveDownloads,
        remote_echoes: &mut RemoteEchoes,
    ) -> Result<()> {
        match event {
            WsEvent::Text(frame) => {
                let outcome = apply_text_event(&frame, vault, store)?;
                debug!(?outcome, "applied session websocket event");
                remote_echoes.record_outcome(&outcome);
                handle_outcome(
                    ws,
                    vault_name,
                    vault,
                    store,
                    &self.config,
                    self.options.transfer,
                    downloads,
                    outcome,
                )
                .await?;
                store.save()?;
            }
            WsEvent::FileChunk(chunk) => {
                downloads.accept_chunk(vault, store, chunk)?;
            }
            WsEvent::Ping(_) | WsEvent::Pong(_) | WsEvent::Binary(_) => {}
            WsEvent::Closed => return Err(SyncSessionError::WebSocketClosed),
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SyncSessionCommand {
    VaultEvent(VaultWatchEvent),
}

#[derive(Debug)]
pub struct SyncSessionOptions {
    pub debounce: Duration,
    pub transfer: TransferOptions,
}

impl SyncSessionOptions {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            debounce: Duration::from_millis(config.daemon.debounce_ms),
            transfer: TransferOptions::new(
                config.sync.transfer_concurrency_enabled,
                config.sync.max_concurrent_transfers,
                config.sync.transfer_timeout_seconds,
            )
            .unwrap_or(TransferOptions {
                concurrency_enabled: true,
                max_concurrent_transfers: 4,
                timeout: Duration::from_secs(60 * 60),
            }),
        }
    }
}

fn handle_command(
    command: Option<SyncSessionCommand>,
    pending_change: &mut Option<Instant>,
    pending_events: &mut PendingWatchEvents,
    remote_echoes: &mut RemoteEchoes,
    debounce: Duration,
) -> Result<()> {
    let Some(command) = command else {
        return Err(SyncSessionError::WebSocketClosed);
    };

    match command {
        SyncSessionCommand::VaultEvent(event) => {
            if remote_echoes.consume(&event) {
                debug!(?event, "ignored remote filesystem echo");
                return Ok(());
            }
            pending_events.push(event);
            *pending_change = Some(Instant::now() + debounce);
        }
    }

    Ok(())
}

fn drain_startup_watch_events(commands: &mut mpsc::Receiver<SyncSessionCommand>) {
    while commands.try_recv().is_ok() {}
}

async fn wait_for_authorization(
    ws: &mut WebSocketClient,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<()> {
    loop {
        match ws.next_event().await? {
            WsEvent::Text(frame) => {
                let outcome = apply_text_event(&frame, vault, store)?;
                if outcome == EventOutcome::AuthorizationAccepted {
                    return Ok(());
                }
            }
            WsEvent::Ping(_) | WsEvent::Pong(_) | WsEvent::Binary(_) | WsEvent::FileChunk(_) => {}
            WsEvent::Closed => return Err(SyncSessionError::WebSocketClosed),
        }
    }
}

async fn send_client_info(
    config: &AppConfig,
    ws: &mut WebSocketClient,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<()> {
    let mut info =
        ClientDescriptor::fnsd(config.client.name.clone(), config.client.version.clone());
    info.protobuf = config.client.protobuf;
    ws.send_client_info(&info).await?;

    if !info.protobuf {
        return Ok(());
    }

    loop {
        match ws.next_event().await? {
            WsEvent::Text(frame) if *frame.action() == Action::ClientInfo => {
                ws.enable_protobuf();
                return Ok(());
            }
            WsEvent::Text(frame) => {
                let _ = apply_text_event(&frame, vault, store)?;
            }
            WsEvent::Ping(_) | WsEvent::Pong(_) | WsEvent::Binary(_) | WsEvent::FileChunk(_) => {}
            WsEvent::Closed => return Err(SyncSessionError::WebSocketClosed),
        }
    }
}

async fn handle_outcome(
    ws: &mut WebSocketClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    transfer: TransferOptions,
    downloads: &mut ActiveDownloads,
    outcome: EventOutcome,
) -> Result<()> {
    match outcome {
        EventOutcome::FileDownloadRequested(download) => {
            let request = build_file_get_request(vault_name, &download);
            ws.send_json(Action::FileChunkDownload, &request).await?;
        }
        EventOutcome::FileDownloadSessionReady(download) => {
            downloads.start(download)?;
        }
        EventOutcome::FileUploadRequested(upload) => {
            send_file_upload(ws, vault, store, &upload, transfer).await?;
        }
        EventOutcome::NoteUploadRequested(resource) => {
            crate::sync::session::local_change::send_local_change(
                ws,
                vault_name,
                vault,
                store,
                config,
                VaultWatchEvent::Changed {
                    path: resource.path.clone(),
                },
            )
            .await?;
        }
        EventOutcome::SettingUploadRequested(path) => {
            crate::sync::session::local_change::send_local_change(
                ws,
                vault_name,
                vault,
                store,
                config,
                VaultWatchEvent::Changed { path },
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}
