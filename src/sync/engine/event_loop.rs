use crate::core::{ResourceKind, VaultName};
use crate::protocol::Action;
use crate::store::LocalStore;
use crate::sync::apply::{EventApplySummary, EventOutcome, SyncEndTracker, apply_text_event};
use crate::sync::transfer::{DownloadSession, build_file_get_request};
use crate::vault::fs::VaultFs;
use crate::ws::{WebSocketClient, WsEvent};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::sync::engine::{
    Result, SyncEngine, SyncEngineError,
    outgoing::{send_file_upload, send_note_modify, send_setting_modify},
    transfer_queue::{QueuedTransfer, TransferKey, TransferState},
};

impl SyncEngine {
    pub(crate) async fn drain_sync_events(
        &self,
        ws: &mut WebSocketClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
    ) -> Result<EventApplySummary> {
        let mut tracker = SyncEndTracker::default();
        let mut summary = EventApplySummary::default();
        let mut transfers = TransferState::new(self.options().transfer);
        let mut pending_server_events = 0_usize;

        while !tracker.is_complete() || pending_server_events > 0 || transfers.has_pending_work()? {
            match self.next_event(ws).await? {
                WsEvent::Text(frame) => {
                    debug!(action = %frame.action().as_str(), "received text event");
                    let outcome = apply_text_event(&frame, vault, store)?;
                    debug!(?outcome, "applied text event");

                    if !matches!(outcome, EventOutcome::SyncEnd { .. }) && pending_server_events > 0
                    {
                        pending_server_events = pending_server_events.saturating_sub(1);
                    }

                    if let EventOutcome::SyncEnd {
                        kind,
                        pending_events,
                        ..
                    } = outcome
                    {
                        info!(?kind, pending_events, "sync end received");
                        tracker.mark(kind);
                        pending_server_events =
                            pending_server_events.saturating_add(pending_events);
                    }

                    self.handle_transfer_outcome(&mut transfers, &outcome);
                    self.release_transfer_slot(&mut transfers, &outcome);
                    self.start_ready_transfers(ws, vault_name, vault, store, &mut transfers)
                        .await?;
                    summary.add(&outcome);
                }
                WsEvent::Ping(_) | WsEvent::Pong(_) => {}
                WsEvent::Binary(frame) => {
                    debug!(prefix = %frame.prefix_str(), "received binary event");
                }
                WsEvent::FileChunk(chunk) => {
                    debug!(
                        session_id = chunk.session_id(),
                        chunk_index = chunk.chunk_index(),
                        bytes = chunk.chunk_data().len(),
                        "received file chunk"
                    );
                    handle_file_chunk(vault, store, &mut transfers, chunk)?;
                    self.start_ready_transfers(ws, vault_name, vault, store, &mut transfers)
                        .await?;
                }
                WsEvent::Closed => return Err(SyncEngineError::WebSocketClosed),
            }
        }

        Ok(summary)
    }

    async fn next_event(&self, ws: &mut WebSocketClient) -> Result<WsEvent> {
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
            EventOutcome::NoteUploadRequested(resource) => {
                debug!(path = %resource.path, "queue note upload");
                transfers.enqueue(QueuedTransfer::NoteUpload(resource.path.clone()));
            }
            EventOutcome::FileUploadRequested(upload) => {
                debug!(path = %upload.path, session_id = %upload.session_id, "queue file upload");
                transfers.enqueue(QueuedTransfer::FileUpload(upload.clone()));
            }
            EventOutcome::FileDownloadRequested(download) => {
                debug!(path = %download.path, "queue file download request");
                transfers.enqueue(QueuedTransfer::FileDownloadRequest(download.clone()));
            }
            EventOutcome::FileDownloadSessionReady(download) => {
                debug!(path = %download.path, session_id = %download.session_id, "queue file download session");
                transfers.enqueue(QueuedTransfer::FileDownloadSession(download.clone()));
            }
            EventOutcome::SettingUploadRequested(path) => {
                debug!(path = %path, "queue setting upload");
                transfers.enqueue(QueuedTransfer::SettingUpload(path.clone()));
            }
            _ => {}
        }
    }

    async fn start_ready_transfers(
        &self,
        ws: &mut WebSocketClient,
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
        ws: &mut WebSocketClient,
        vault_name: &VaultName,
        vault: &VaultFs,
        store: &mut LocalStore,
        transfers: &mut TransferState,
        transfer: QueuedTransfer,
    ) -> Result<()> {
        let key = transfer.key();
        debug!(?key, "starting transfer");

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
                debug!(path = %download.path, "requesting file download");
                ws.send_json(Action::FileChunkDownload, &request).await?;
                transfers.track_download_request(&download, key);
            }
            QueuedTransfer::FileDownloadSession(download) => {
                let mut session = DownloadSession::new(download)?;
                store.restore_download_chunks(&mut session)?;
                debug!(
                    path = %session.path(),
                    session_id = session.session_id(),
                    received = session.received_chunks(),
                    total = session.total_chunks(),
                    "restored download checkpoint"
                );

                if session.is_complete() {
                    finalize_download(vault, store, session)?;
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
                debug!(?kind, path = %path, "transfer ack received");
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
    chunk: crate::protocol::FileChunkFrame,
) -> Result<()> {
    let session_id = chunk.session_id().to_string();
    let Some(session) = transfers.download_mut(&session_id) else {
        warn!(session_id = %session_id, "ignored file chunk without active download session");
        return Ok(());
    };

    let chunk_index = chunk.chunk_index();
    let chunk_data = chunk.chunk_data().to_vec();
    session.accept_chunk(chunk)?;
    store.save_download_chunk(session, chunk_index, &chunk_data)?;

    if session.is_complete() {
        let session = transfers
            .take_download(&session_id)
            .expect("download session exists after successful chunk accept");
        finalize_download(vault, store, session)?;
        info!(session_id = %session_id, "file download completed");
        transfers.finish(TransferKey::DownloadSession(session_id));
    }

    Ok(())
}

fn finalize_download(
    vault: &VaultFs,
    store: &mut LocalStore,
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
    )?;
    store.clear_download_chunks(&content_hash, size, chunk_size)?;
    Ok(())
}
