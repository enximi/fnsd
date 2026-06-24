use std::{collections::BTreeSet, time::Duration};

use fns_config::AppConfig;
use fns_core::{ResourceKind, VaultName, VaultPath};
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

use crate::{Result, SyncSessionError, local_change::send_local_changes};

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
        ws: &mut FnsWsClient,
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

#[derive(Debug, Default)]
struct RemoteEchoes {
    renames: Vec<EchoRename>,
}

impl RemoteEchoes {
    fn record_outcome(&mut self, outcome: &EventOutcome) {
        match outcome {
            EventOutcome::RemoteWrite { .. }
            | EventOutcome::RemoteDelete { .. }
            | EventOutcome::RemoteMtimeUpdate { .. } => {}
            EventOutcome::RemoteRename {
                kind,
                old_path,
                new_path,
            } => {
                self.record_rename(*kind, old_path.clone(), new_path.clone());
            }
            EventOutcome::AuthorizationAccepted
            | EventOutcome::Ack { .. }
            | EventOutcome::SyncEnd { .. }
            | EventOutcome::NeedNoteUpload(_)
            | EventOutcome::NeedFileUpload(_)
            | EventOutcome::NeedFileDownload(_)
            | EventOutcome::NeedFileDownloadSession(_)
            | EventOutcome::NeedSettingUpload(_)
            | EventOutcome::Ignored => {}
        }
    }

    fn record_rename(&mut self, kind: ResourceKind, old_path: VaultPath, new_path: VaultPath) {
        self.renames.push(EchoRename {
            kind,
            old_path,
            new_path,
            old_seen: false,
            new_seen: false,
            expires_at: echo_expires_at(),
        });
    }

    fn consume(&mut self, event: &VaultWatchEvent) -> bool {
        self.prune_expired();

        match event {
            VaultWatchEvent::Changed { path } => self.consume_rename_changed(path),
            VaultWatchEvent::RenameFrom { path } => self.consume_rename_from(path),
            VaultWatchEvent::RenameTo { path } => self.consume_rename_to(path),
            VaultWatchEvent::Renamed { old_path, new_path } => {
                self.consume_rename(old_path, new_path)
            }
            VaultWatchEvent::RescanNeeded => false,
        }
    }

    fn consume_rename_changed(&mut self, path: &VaultPath) -> bool {
        let Some(index) = self
            .renames
            .iter_mut()
            .position(|echo| echo.mark_changed(path))
        else {
            return false;
        };
        if self.renames[index].is_complete() {
            self.renames.remove(index);
        }
        true
    }

    fn consume_rename_from(&mut self, path: &VaultPath) -> bool {
        self.consume_rename_side(path, RenameEchoSide::Old)
    }

    fn consume_rename_to(&mut self, path: &VaultPath) -> bool {
        self.consume_rename_side(path, RenameEchoSide::New)
    }

    fn consume_rename_side(&mut self, path: &VaultPath, side: RenameEchoSide) -> bool {
        let Some(index) = self
            .renames
            .iter_mut()
            .position(|echo| echo.mark_side(path, side))
        else {
            return false;
        };
        if self.renames[index].is_complete() {
            self.renames.remove(index);
        }
        true
    }

    fn consume_rename(&mut self, old_path: &VaultPath, new_path: &VaultPath) -> bool {
        let Some(index) = self
            .renames
            .iter()
            .position(|echo| echo.matches_rename(old_path, new_path))
        else {
            return false;
        };
        self.renames.remove(index);
        true
    }

    fn prune_expired(&mut self) {
        let now = Instant::now();
        self.renames.retain(|echo| echo.expires_at > now);
    }
}

#[derive(Debug)]
struct EchoRename {
    kind: ResourceKind,
    old_path: VaultPath,
    new_path: VaultPath,
    old_seen: bool,
    new_seen: bool,
    expires_at: Instant,
}

impl EchoRename {
    fn mark_changed(&mut self, path: &VaultPath) -> bool {
        let old_matches = paths_match(self.kind, &self.old_path, path);
        let new_matches = paths_match(self.kind, &self.new_path, path);
        self.old_seen |= old_matches;
        self.new_seen |= new_matches;
        old_matches || new_matches
    }

    fn mark_side(&mut self, path: &VaultPath, side: RenameEchoSide) -> bool {
        let matches = match side {
            RenameEchoSide::Old => paths_match(self.kind, &self.old_path, path),
            RenameEchoSide::New => paths_match(self.kind, &self.new_path, path),
        };
        if !matches {
            return false;
        }

        match side {
            RenameEchoSide::Old => self.old_seen = true,
            RenameEchoSide::New => self.new_seen = true,
        }
        true
    }

    fn matches_rename(&self, old_path: &VaultPath, new_path: &VaultPath) -> bool {
        paths_match(self.kind, &self.old_path, old_path)
            && paths_match(self.kind, &self.new_path, new_path)
    }

    fn is_complete(&self) -> bool {
        self.old_seen && self.new_seen
    }
}

#[derive(Debug, Clone, Copy)]
enum RenameEchoSide {
    Old,
    New,
}

fn paths_match(kind: ResourceKind, expected: &VaultPath, actual: &VaultPath) -> bool {
    expected == actual || (kind == ResourceKind::Folder && is_child_path(expected, actual))
}

fn is_child_path(parent: &VaultPath, path: &VaultPath) -> bool {
    path.as_str()
        .strip_prefix(parent.as_str())
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn echo_expires_at() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

#[derive(Debug, Default)]
struct PendingWatchEvents {
    paths: BTreeSet<fns_core::VaultPath>,
    rename_from: Vec<fns_core::VaultPath>,
    rename_to: Vec<fns_core::VaultPath>,
    renames: Vec<(fns_core::VaultPath, fns_core::VaultPath)>,
    rescan_needed: bool,
}

impl PendingWatchEvents {
    fn push(&mut self, event: VaultWatchEvent) {
        match event {
            VaultWatchEvent::Changed { path } => {
                if self
                    .renames
                    .iter()
                    .any(|(old_path, new_path)| old_path == &path || new_path == &path)
                {
                    return;
                }
                self.paths.insert(path);
            }
            VaultWatchEvent::RenameFrom { path } => {
                if self.renames.iter().any(|(old_path, _)| old_path == &path) {
                    return;
                }
                self.paths.remove(&path);
                if let Some(new_path) = pop_front(&mut self.rename_to) {
                    self.push_rename(path, new_path);
                } else {
                    self.rename_from.push(path);
                }
            }
            VaultWatchEvent::RenameTo { path } => {
                if self.renames.iter().any(|(_, new_path)| new_path == &path) {
                    return;
                }
                self.paths.remove(&path);
                if let Some(old_path) = pop_front(&mut self.rename_from) {
                    self.push_rename(old_path, path);
                } else {
                    self.rename_to.push(path);
                }
            }
            VaultWatchEvent::Renamed { old_path, new_path } => {
                self.paths.remove(&old_path);
                self.paths.remove(&new_path);
                self.rename_from.retain(|path| path != &old_path);
                self.rename_to.retain(|path| path != &new_path);
                self.push_rename(old_path, new_path);
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
            std::mem::take(&mut self.renames)
                .into_iter()
                .map(|(old_path, new_path)| VaultWatchEvent::Renamed { old_path, new_path }),
        );
        events.extend(
            std::mem::take(&mut self.rename_from)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events.extend(
            std::mem::take(&mut self.rename_to)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events.extend(
            std::mem::take(&mut self.paths)
                .into_iter()
                .map(|path| VaultWatchEvent::Changed { path }),
        );
        events
    }

    fn push_rename(&mut self, old_path: fns_core::VaultPath, new_path: fns_core::VaultPath) {
        if self.renames.iter().any(|(existing_old, existing_new)| {
            existing_old == &old_path && existing_new == &new_path
        }) {
            return;
        }

        self.renames.push((old_path, new_path));
    }
}

fn pop_front<T>(items: &mut Vec<T>) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        Some(items.remove(0))
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
