use crate::core::ResourceKind;
use crate::store::LocalStore;
use crate::sync::engine::TransferOptions;
use crate::sync::plan::{FileDownload, FileUpload};
use crate::sync::transfer::{DownloadSession, build_upload_plan};
use crate::vault::fs::VaultFs;
use crate::ws::WebSocketClient;
use tracing::warn;

use crate::sync::session::{Result, SyncSessionError};

#[derive(Debug, Default)]
pub(crate) struct ActiveDownloads {
    sessions: std::collections::BTreeMap<String, DownloadSession>,
}

impl ActiveDownloads {
    pub(crate) fn start(&mut self, download: FileDownload) -> Result<()> {
        let session = DownloadSession::new(download)?;
        warn!(
            path = %session.path(),
            session_id = session.session_id(),
            "started long-lived file download session"
        );
        self.sessions
            .insert(session.session_id().to_string(), session);
        Ok(())
    }

    pub(crate) fn accept_chunk(
        &mut self,
        vault: &VaultFs,
        store: &mut LocalStore,
        chunk: crate::protocol::FileChunkFrame,
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

pub(crate) async fn send_file_upload(
    ws: &mut WebSocketClient,
    vault: &VaultFs,
    store: &mut LocalStore,
    upload: &FileUpload,
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

    store.set_pending_modify(ResourceKind::File, &plan.path, &plan.content_hash);
    store.set_content_hash(
        ResourceKind::File,
        &plan.path,
        Some(plan.content_hash),
        plan.mtime,
        plan.size,
    );
    store.save()?;
    Ok(())
}
