use fns_core::{ResourceKind, VaultName};
use fns_file_transfer::{DownloadSession, build_file_get_request};
use fns_local_store::LocalStore;
use fns_protocol::Action;
use fns_sync_apply::{EventApplySummary, EventOutcome, SyncEndTracker, apply_text_event};
use fns_vault_fs::VaultFs;
use fns_ws_client::{FnsWsClient, WsEvent};
use tokio::time::timeout;

use crate::{
    Result, SyncEngine, SyncEngineError,
    checkpoint::DownloadCheckpointStore,
    outgoing::{send_file_upload, send_note_modify, send_setting_modify},
    transfer::{QueuedTransfer, TransferKey, TransferState},
};

impl SyncEngine {
    pub(crate) async fn drain_sync_events(
        &self,
        ws: &mut FnsWsClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
    ) -> Result<EventApplySummary> {
        let mut tracker = SyncEndTracker::default();
        let mut summary = EventApplySummary::default();
        let mut transfers = TransferState::new(self.options().transfer);
        let checkpoints = DownloadCheckpointStore::new(store.path());

        while !tracker.is_complete() || transfers.has_pending_work()? {
            match self.next_event(ws).await? {
                WsEvent::Text(frame) => {
                    let outcome = apply_text_event(&frame, vault, store)?;

                    if let EventOutcome::SyncEnd { kind, .. } = outcome {
                        tracker.mark(kind);
                    }

                    self.handle_transfer_outcome(&mut transfers, &outcome);
                    self.release_transfer_slot(&mut transfers, &outcome);
                    self.start_ready_transfers(ws, vault_name, vault, store, &mut transfers)
                        .await?;
                    summary.add(&outcome);
                }
                WsEvent::Ping(_) | WsEvent::Pong(_) => {}
                WsEvent::Binary(_) => {}
                WsEvent::FileChunk(chunk) => {
                    handle_file_chunk(vault, store, &mut transfers, &checkpoints, chunk)?;
                    self.start_ready_transfers(ws, vault_name, vault, store, &mut transfers)
                        .await?;
                }
                WsEvent::Closed => return Err(SyncEngineError::WebSocketClosed),
            }
        }

        Ok(summary)
    }

    async fn next_event(&self, ws: &mut FnsWsClient) -> Result<WsEvent> {
        let wait = self.options().transfer.timeout;
        if wait.is_zero() {
            return Ok(ws.next_event().await?);
        }

        timeout(wait, ws.next_event())
            .await
            .map_err(|_| SyncEngineError::TransferTimeout("waiting for websocket event".into()))?
            .map_err(Into::into)
    }

    fn handle_transfer_outcome(&self, transfers: &mut TransferState, outcome: &EventOutcome) {
        match outcome {
            EventOutcome::NeedNoteUpload(resource) => {
                transfers.enqueue(QueuedTransfer::NoteUpload(resource.path.clone()));
            }
            EventOutcome::NeedFileUpload(upload) => {
                transfers.enqueue(QueuedTransfer::FileUpload(upload.clone()));
            }
            EventOutcome::NeedFileDownload(download) => {
                transfers.enqueue(QueuedTransfer::FileDownloadRequest(download.clone()));
            }
            EventOutcome::NeedFileDownloadSession(download) => {
                transfers.enqueue(QueuedTransfer::FileDownloadSession(download.clone()));
            }
            EventOutcome::NeedSettingUpload(path) => {
                transfers.enqueue(QueuedTransfer::SettingUpload(path.clone()));
            }
            _ => {}
        }
    }

    async fn start_ready_transfers(
        &self,
        ws: &mut FnsWsClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        transfers: &mut TransferState,
    ) -> Result<()> {
        while let Some(transfer) = transfers.next_ready()? {
            self.start_transfer(ws, vault_name, vault, store, transfers, transfer)
                .await?;
        }

        Ok(())
    }

    async fn start_transfer(
        &self,
        ws: &mut FnsWsClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        transfers: &mut TransferState,
        transfer: QueuedTransfer,
    ) -> Result<()> {
        let key = transfer.key();

        match transfer {
            QueuedTransfer::NoteUpload(path) => {
                send_note_modify(ws, vault_name, vault, store, &path).await?;
            }
            QueuedTransfer::FileUpload(upload) => {
                send_file_upload(ws, vault, store, &upload, self.options().transfer.timeout)
                    .await?;
            }
            QueuedTransfer::FileDownloadRequest(download) => {
                let request = build_file_get_request(vault_name, &download);
                ws.send_json(Action::FileChunkDownload, &request).await?;
                transfers.track_download_request(&download, key);
            }
            QueuedTransfer::FileDownloadSession(download) => {
                let mut session = DownloadSession::new(download)?;
                let checkpoints = DownloadCheckpointStore::new(store.path());
                checkpoints.restore(&mut session)?;

                if session.is_complete() {
                    finalize_download(vault, store, &checkpoints, session)?;
                    transfers.finish(key);
                } else {
                    transfers.insert_download(session);
                }
            }
            QueuedTransfer::SettingUpload(path) => {
                send_setting_modify(ws, vault_name, vault, store, &path).await?;
            }
        }

        Ok(())
    }

    fn release_transfer_slot(&self, transfers: &mut TransferState, outcome: &EventOutcome) {
        match outcome {
            EventOutcome::Ack { kind, path } if *kind != ResourceKind::Folder => {
                transfers.finish(TransferKey::Resource(*kind, path.clone()));
            }
            _ => {}
        }
    }
}

fn handle_file_chunk(
    vault: &VaultFs,
    store: &mut LocalStore,
    transfers: &mut TransferState,
    checkpoints: &DownloadCheckpointStore,
    chunk: fns_protocol::FileChunkFrame,
) -> Result<()> {
    let session_id = chunk.session_id().to_string();
    let Some(session) = transfers.download_mut(&session_id) else {
        return Ok(());
    };

    checkpoints.save_chunk(session, chunk.chunk_index(), chunk.chunk_data())?;
    session.accept_chunk(chunk)?;

    if session.is_complete() {
        let session = transfers
            .take_download(&session_id)
            .expect("download session exists after successful chunk accept");
        finalize_download(vault, store, checkpoints, session)?;
        transfers.finish(TransferKey::DownloadSession(session_id));
    }

    Ok(())
}

fn finalize_download(
    vault: &VaultFs,
    store: &mut LocalStore,
    checkpoints: &DownloadCheckpointStore,
    session: DownloadSession,
) -> Result<()> {
    let content_hash = session.content_hash().clone();
    let size = session.size();
    let chunk_size = session.chunk_size();
    let file = session.finalize(vault)?;
    store.set_content_hash(
        ResourceKind::File,
        &file.path,
        Some(file.content_hash),
        file.mtime,
        file.size,
    );
    checkpoints.clear_completed(&content_hash, size, chunk_size)?;
    Ok(())
}
