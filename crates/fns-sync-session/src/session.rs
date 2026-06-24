use std::{collections::BTreeSet, time::Duration};

use fns_config::AppConfig;
use fns_core::{ResourceKind, VaultName};
use fns_file_transfer::{DownloadSession, build_file_get_request, build_upload_plan};
use fns_local_store::LocalStore;
use fns_protocol::Action;
use fns_sync_apply::{EventOutcome, apply_text_event};
use fns_sync_engine::{SyncEngine, TransferOptions};
use fns_vault_fs::VaultFs;
use fns_vault_watch::VaultWatchEvent;
use fns_ws_client::{ClientInfo, FnsWsClient, WsEvent};
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep_until};
use tracing::{debug, info, warn};

use crate::{Result, SyncSessionError, local_change::send_local_change};

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

    pub fn with_options(config: AppConfig, options: SyncSessionOptions) -> Self {
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
                            handle_command(command, &mut pending_change, &mut pending_events, self.options.debounce)?;
                        }
                        event = ws.next_event() => {
                            self.handle_ws_event(event?, &mut ws, &vault_name, &vault, &mut store, &mut downloads).await?;
                        }
                    }
                }
                None => {
                    tokio::select! {
                        command = commands.recv() => {
                            handle_command(command, &mut pending_change, &mut pending_events, self.options.debounce)?;
                        }
                        event = ws.next_event() => {
                            self.handle_ws_event(event?, &mut ws, &vault_name, &vault, &mut store, &mut downloads).await?;
                        }
                    }
                }
            }
        }
    }

    async fn startup_sync(
        &self,
        ws: &mut FnsWsClient,
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

    async fn connect(&self, vault: &VaultFs, store: &mut LocalStore) -> Result<FnsWsClient> {
        let ws_url = self
            .config
            .server
            .ws_url_with_protocol(self.config.client.protobuf)?;
        info!(server = %ws_url, "connecting long-lived websocket session");
        let mut ws = FnsWsClient::connect(&ws_url).await?;
        ws.authorize(self.config.server.api_token.clone()).await?;
        wait_for_authorization(&mut ws, vault, store).await?;
        send_client_info(&self.config, &mut ws, vault, store).await?;
        Ok(ws)
    }

    async fn handle_pending_changes(
        &self,
        ws: &mut FnsWsClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        pending_events: &mut PendingWatchEvents,
    ) -> Result<()> {
        for change in pending_events.take_all() {
            send_local_change(ws, vault_name, vault, store, &self.config, change).await?;
        }
        Ok(())
    }

    async fn handle_ws_event(
        &self,
        event: WsEvent,
        ws: &mut FnsWsClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        downloads: &mut ActiveDownloads,
    ) -> Result<()> {
        match event {
            WsEvent::Text(frame) => {
                let outcome = apply_text_event(&frame, vault, store)?;
                debug!(?outcome, "applied session websocket event");
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
    debounce: Duration,
) -> Result<()> {
    let Some(command) = command else {
        return Err(SyncSessionError::WebSocketClosed);
    };

    match command {
        SyncSessionCommand::VaultEvent(event) => {
            pending_events.push(event);
            *pending_change = Some(Instant::now() + debounce);
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
struct PendingWatchEvents {
    paths: BTreeSet<fns_core::VaultPath>,
    rescan_needed: bool,
}

impl PendingWatchEvents {
    fn push(&mut self, event: VaultWatchEvent) {
        match event {
            VaultWatchEvent::Changed { path } => {
                self.paths.insert(path);
            }
            VaultWatchEvent::RescanNeeded => {
                self.rescan_needed = true;
            }
        }
    }

    fn take_all(&mut self) -> Vec<VaultWatchEvent> {
        let mut events = Vec::new();
        if self.rescan_needed {
            events.push(VaultWatchEvent::RescanNeeded);
            self.rescan_needed = false;
        }

        events.extend(
            std::mem::take(&mut self.paths)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events
    }
}

fn drain_startup_watch_events(commands: &mut mpsc::Receiver<SyncSessionCommand>) {
    while commands.try_recv().is_ok() {}
}

async fn wait_for_authorization(
    ws: &mut FnsWsClient,
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
    ws: &mut FnsWsClient,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<()> {
    let mut info = ClientInfo::headless(config.client.name.clone(), config.client.version.clone());
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
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    config: &AppConfig,
    transfer: TransferOptions,
    downloads: &mut ActiveDownloads,
    outcome: EventOutcome,
) -> Result<()> {
    match outcome {
        EventOutcome::NeedFileDownload(download) => {
            let request = build_file_get_request(vault_name, &download);
            ws.send_json(Action::FileChunkDownload, &request).await?;
        }
        EventOutcome::NeedFileDownloadSession(download) => {
            let session = DownloadSession::new(download)?;
            warn!(
                path = %session.path(),
                session_id = session.session_id(),
                "started long-lived file download session"
            );
            downloads.insert(session);
        }
        EventOutcome::NeedFileUpload(upload) => {
            send_file_upload(ws, vault, store, &upload, transfer).await?;
        }
        EventOutcome::NeedNoteUpload(resource) => {
            crate::local_change::send_local_change(
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
        EventOutcome::NeedSettingUpload(path) => {
            crate::local_change::send_local_change(
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

#[derive(Debug, Default)]
struct ActiveDownloads {
    sessions: std::collections::BTreeMap<String, DownloadSession>,
}

impl ActiveDownloads {
    fn insert(&mut self, session: DownloadSession) {
        self.sessions
            .insert(session.session_id().to_string(), session);
    }

    fn accept_chunk(
        &mut self,
        vault: &VaultFs,
        store: &mut LocalStore,
        chunk: fns_protocol::FileChunkFrame,
    ) -> Result<()> {
        let session_id = chunk.session_id().to_string();
        let Some(session) = self.sessions.get_mut(&session_id) else {
            warn!(session_id = %session_id, "ignored file chunk without active session");
            return Ok(());
        };

        session.accept_chunk(chunk)?;

        if session.is_complete() {
            let session = self
                .sessions
                .remove(&session_id)
                .expect("download session exists after chunk accept");
            let file = session.finalize(vault)?;
            store.set_content_hash(
                ResourceKind::File,
                &file.path,
                Some(file.content_hash),
                file.mtime,
                file.size,
            );
            store.save()?;
        }

        Ok(())
    }
}

async fn send_file_upload(
    ws: &mut FnsWsClient,
    vault: &VaultFs,
    store: &mut LocalStore,
    upload: &fns_sync_plan::FileUpload,
    transfer: TransferOptions,
) -> Result<()> {
    let plan = build_upload_plan(vault, upload)?;
    for chunk in &plan.chunks {
        if transfer.timeout.is_zero() {
            ws.send_file_chunk(chunk).await?;
        } else {
            tokio::time::timeout(transfer.timeout, ws.send_file_chunk(chunk))
                .await
                .map_err(|_| SyncSessionError::WebSocketClosed)??;
        }
    }

    store.set_pending_modify(fns_core::ResourceKind::File, &plan.path, &plan.content_hash);
    store.set_content_hash(
        fns_core::ResourceKind::File,
        &plan.path,
        Some(plan.content_hash),
        plan.mtime,
        plan.size,
    );
    store.save()?;
    Ok(())
}
