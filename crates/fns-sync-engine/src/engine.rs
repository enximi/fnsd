use fns_config::AppConfig;
use fns_core::VaultName;
use fns_local_store::LocalStore;
use fns_protocol::Action;
use fns_sync_apply::{EventApplySummary, EventOutcome, apply_text_event, refresh_sent_hash_index};
use fns_sync_plan::{
    build_file_sync_request, build_folder_sync_request, build_note_sync_request,
    build_setting_sync_request,
};
use fns_vault_fs::{VaultFs, VaultScanOptions};
use fns_ws_client::{ClientInfo, FnsWsClient, WsEvent};
use tracing::{debug, info};

use crate::{Result, SyncEngineError, snapshot::SyncBatches, transfer::TransferOptions};

#[derive(Debug)]
pub struct SyncEngine {
    config: AppConfig,
    options: SyncEngineOptions,
}

impl SyncEngine {
    pub fn new(config: AppConfig) -> Self {
        let options = SyncEngineOptions::from_config(&config);
        Self { config, options }
    }

    pub fn with_options(config: AppConfig, options: SyncEngineOptions) -> Self {
        Self { config, options }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub(crate) fn options(&self) -> &SyncEngineOptions {
        &self.options
    }

    pub async fn sync_once(&self) -> Result<SyncOnceSummary> {
        info!(vault_root = %self.config.vault.root.display(), "starting sync once");
        let vault = VaultFs::new(&self.config.vault.root)?;
        let mut store = LocalStore::open(&self.config.store.path)?;
        let ws_url = self
            .config
            .server
            .ws_url_with_protocol(self.config.client.protobuf)?;
        info!(server = %ws_url, "connecting websocket");
        let mut ws = FnsWsClient::connect(&ws_url).await?;
        debug!("sending authorization request");
        ws.authorize(self.config.server.api_token.clone()).await?;
        self.wait_for_authorization(&mut ws, &vault, &mut store)
            .await?;
        info!("authorization accepted");
        self.send_client_info(&mut ws, &vault, &mut store).await?;

        let summary = self.sync_authenticated(&mut ws, &vault, &mut store).await?;
        store.save()?;
        Ok(summary)
    }

    pub async fn sync_authenticated(
        &self,
        ws: &mut FnsWsClient,
        vault: &VaultFs,
        store: &mut LocalStore,
    ) -> Result<SyncOnceSummary> {
        let vault_name = self.config.vault_name()?;
        let scan_options = self.config.scan_options()?;
        debug!("scanning vault");
        let snapshot = scan_vault(self.config.vault.root.clone(), scan_options).await?;
        let context = Some(uuid::Uuid::new_v4().to_string());
        let batches =
            SyncBatches::from_snapshot(snapshot, store, context, self.options.missing_path_mode)?;
        info!(
            notes = batches.notes.items.len(),
            files = batches.files.items.len(),
            folders = batches.folders.items.len(),
            settings = batches.settings.items.len(),
            "prepared local sync batches"
        );

        self.send_sync_requests(ws, &vault_name, &batches).await?;

        refresh_sent_hash_index(
            store,
            &batches.notes,
            &batches.files,
            &batches.folders,
            &batches.settings,
        );
        let events = self
            .drain_sync_events(ws, &vault_name, vault, store)
            .await?;
        info!(
            events = events.text_events,
            remote_writes = events.remote_writes,
            remote_deletes = events.remote_deletes,
            remote_renames = events.remote_renames,
            acks = events.acks,
            "sync once completed"
        );

        Ok(SyncOnceSummary {
            notes_sent: batches.notes.items.len(),
            files_sent: batches.files.items.len(),
            folders_sent: batches.folders.items.len(),
            settings_sent: batches.settings.items.len(),
            events,
        })
    }

    fn client_info(&self) -> ClientInfo {
        let mut info = ClientInfo::headless(
            self.config.client.name.clone(),
            self.config.client.version.clone(),
        );
        info.protobuf = self.config.client.protobuf;
        info
    }

    async fn send_client_info(
        &self,
        ws: &mut FnsWsClient,
        vault: &VaultFs,
        store: &mut LocalStore,
    ) -> Result<()> {
        let info = self.client_info();
        ws.send_client_info(&info).await?;

        if !info.protobuf {
            return Ok(());
        }

        loop {
            match ws.next_event().await? {
                WsEvent::Text(frame) if *frame.action() == Action::ClientInfo => {
                    ws.enable_protobuf();
                    debug!("protobuf protocol enabled");
                    return Ok(());
                }
                WsEvent::Text(frame) => {
                    let outcome = apply_text_event(&frame, vault, store)?;
                    debug!(?outcome, "applied event while waiting for client info");
                }
                WsEvent::Ping(_) | WsEvent::Pong(_) => {}
                WsEvent::Binary(_) | WsEvent::FileChunk(_) => {}
                WsEvent::Closed => return Err(SyncEngineError::WebSocketClosed),
            }
        }
    }

    async fn send_sync_requests(
        &self,
        ws: &mut FnsWsClient,
        vault_name: &VaultName,
        batches: &SyncBatches,
    ) -> Result<()> {
        let folder_sync = build_folder_sync_request(vault_name, &batches.folders);
        debug!(
            items = batches.folders.items.len(),
            "sending folder sync request"
        );
        ws.send_json(Action::FolderSync, &folder_sync).await?;

        let note_sync = build_note_sync_request(vault_name, &batches.notes);
        debug!(
            items = batches.notes.items.len(),
            "sending note sync request"
        );
        ws.send_json(Action::NoteSync, &note_sync).await?;

        let file_sync = build_file_sync_request(vault_name, &batches.files)?;
        debug!(
            items = batches.files.items.len(),
            "sending file sync request"
        );
        ws.send_json(Action::FileSync, &file_sync).await?;

        let setting_sync = build_setting_sync_request(vault_name, &batches.settings);
        debug!(
            items = batches.settings.items.len(),
            "sending setting sync request"
        );
        ws.send_json(Action::SettingSync, &setting_sync).await?;

        Ok(())
    }

    async fn wait_for_authorization(
        &self,
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
                WsEvent::Ping(_) | WsEvent::Pong(_) => {}
                WsEvent::Binary(_) | WsEvent::FileChunk(_) => {}
                WsEvent::Closed => return Err(SyncEngineError::WebSocketClosed),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncEngineOptions {
    pub missing_path_mode: MissingPathMode,
    pub transfer: TransferOptions,
}

impl Default for SyncEngineOptions {
    fn default() -> Self {
        Self {
            missing_path_mode: MissingPathMode::Missing,
            transfer: TransferOptions {
                concurrency_enabled: true,
                max_concurrent_transfers: 4,
                timeout: std::time::Duration::from_secs(60 * 60),
            },
        }
    }
}

impl SyncEngineOptions {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            missing_path_mode: if config.sync.offline_delete_sync_enabled {
                MissingPathMode::Deleted
            } else {
                MissingPathMode::Missing
            },
            transfer: TransferOptions::new(
                config.sync.transfer_concurrency_enabled,
                config.sync.max_concurrent_transfers,
                config.sync.transfer_timeout_seconds,
            )
            .unwrap_or_else(|_| Self::default().transfer),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingPathMode {
    Missing,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOnceSummary {
    pub notes_sent: usize,
    pub files_sent: usize,
    pub folders_sent: usize,
    pub settings_sent: usize,
    pub events: EventApplySummary,
}

async fn scan_vault(
    root: std::path::PathBuf,
    options: VaultScanOptions,
) -> Result<fns_vault_fs::VaultSnapshot> {
    tokio::task::spawn_blocking(move || {
        let vault = VaultFs::new(root)?;
        vault.scan(&options)
    })
    .await
    .map_err(|err| SyncEngineError::BlockingTask(err.to_string()))?
    .map_err(Into::into)
}
