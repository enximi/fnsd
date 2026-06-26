use crate::core::{ResourceKind, VaultName, VaultPath};
use crate::hash::{setting_content_hash, text_content_hash};
use crate::protocol::Action;
use crate::store::{LocalStore, UploadCheckpoint};
use crate::sync::plan::{FileUpload, build_note_modify_request, build_setting_modify_request};
use crate::sync::transfer::build_upload_plan;
use crate::vault::fs::VaultFs;
use crate::ws::WebSocketClient;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info};

use crate::sync::engine::{Result, SyncEngineError};

pub(crate) async fn send_note_modify(
    ws: &mut WebSocketClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    debug!(path = %path, "sending note modify");
    let content = vault.read_text(path)?;
    let metadata = vault.file_metadata(path)?;
    let content_hash = text_content_hash(&content);
    let request = build_note_modify_request(
        vault_name,
        path,
        content,
        &content_hash,
        metadata.ctime,
        metadata.mtime,
    )?;

    ws.send_json(Action::NoteModify, &request).await?;
    store.set_pending_modify(ResourceKind::Note, path, &content_hash)?;
    store.set_content_hash(
        ResourceKind::Note,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    )?;
    Ok(())
}

pub(crate) async fn send_setting_modify(
    ws: &mut WebSocketClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    debug!(path = %path, "sending setting modify");
    let content = vault.read_text(path)?;
    let metadata = vault.file_metadata(path)?;
    let content_hash = setting_content_hash(content.as_bytes());
    let request = build_setting_modify_request(
        vault_name,
        path,
        content,
        &content_hash,
        metadata.ctime,
        metadata.mtime,
    )?;

    ws.send_json(Action::SettingModify, &request).await?;
    store.set_pending_modify(ResourceKind::Setting, path, &content_hash)?;
    store.set_content_hash(
        ResourceKind::Setting,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    )?;
    Ok(())
}

pub(crate) async fn send_file_upload(
    ws: &mut WebSocketClient,
    vault: &VaultFs,
    store: &mut LocalStore,
    upload: &FileUpload,
    transfer_timeout: Duration,
) -> Result<()> {
    let plan = build_upload_plan(vault, upload)?;
    let start_chunk_index = resume_start_chunk(store, &plan)?;
    info!(
        path = %plan.path,
        session_id = %plan.session_id,
        start_chunk = start_chunk_index,
        total_chunks = plan.total_chunks,
        size = plan.size,
        "sending file upload"
    );

    for chunk in &plan.chunks {
        if chunk.chunk_index() < start_chunk_index {
            continue;
        }

        debug!(
            path = %plan.path,
            session_id = %plan.session_id,
            chunk_index = chunk.chunk_index(),
            total_chunks = plan.total_chunks,
            bytes = chunk.chunk_data().len(),
            "sending file chunk"
        );
        send_file_chunk(ws, chunk, transfer_timeout).await?;
        if (chunk.chunk_index() as usize) + 1 < plan.total_chunks {
            store.set_file_upload_checkpoint(
                &plan.path,
                UploadCheckpoint::new(
                    &plan.session_id,
                    plan.content_hash.clone(),
                    chunk.chunk_index(),
                ),
            )?;
        }
    }

    store.set_pending_modify(ResourceKind::File, &plan.path, &plan.content_hash)?;
    store.set_content_hash(
        ResourceKind::File,
        &plan.path,
        Some(plan.content_hash),
        plan.mtime,
        plan.size,
    )?;
    debug!(path = %plan.path, "file upload sent and pending state saved");
    Ok(())
}

async fn send_file_chunk(
    ws: &mut WebSocketClient,
    chunk: &crate::protocol::FileChunkFrame,
    transfer_timeout: Duration,
) -> Result<()> {
    if transfer_timeout.is_zero() {
        return Ok(ws.send_file_chunk(chunk).await?);
    }

    timeout(transfer_timeout, ws.send_file_chunk(chunk))
        .await
        .map_err(|_| {
            SyncEngineError::TransferTimeout(format!(
                "upload chunk {} for session {}",
                chunk.chunk_index(),
                chunk.session_id()
            ))
        })?
        .map_err(Into::into)
}

fn resume_start_chunk(store: &LocalStore, plan: &crate::sync::transfer::UploadPlan) -> Result<u32> {
    let Some(checkpoint) = store.file_upload_checkpoint(&plan.path)? else {
        return Ok(0);
    };

    if checkpoint.session_id != plan.session_id {
        debug!(
            path = %plan.path,
            checkpoint_session_id = %checkpoint.session_id,
            upload_session_id = %plan.session_id,
            "upload checkpoint ignored because session changed"
        );
        return Ok(0);
    }

    if checkpoint.content_hash()?.as_str() != plan.content_hash.as_str() {
        debug!(path = %plan.path, "upload checkpoint ignored because content hash changed");
        return Ok(0);
    }

    let start_chunk_index = checkpoint.last_chunk_index.saturating_add(1);
    if start_chunk_index as usize > plan.total_chunks {
        return Err(SyncEngineError::UploadCheckpointOutOfRange {
            path: plan.path.clone(),
            index: checkpoint.last_chunk_index,
            total: plan.total_chunks,
        });
    }

    Ok(start_chunk_index)
}
