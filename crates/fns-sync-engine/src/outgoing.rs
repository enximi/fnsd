use fns_core::{ResourceKind, VaultName, VaultPath};
use fns_file_transfer::build_upload_plan;
use fns_hash::text_content_hash;
use fns_local_store::{LocalStore, UploadCheckpoint};
use fns_protocol::Action;
use fns_sync_plan::{FileUpload, build_note_modify_request, build_setting_modify_request};
use fns_vault_fs::VaultFs;
use fns_ws_client::FnsWsClient;
use std::time::Duration;
use tokio::time::timeout;

use crate::{Result, SyncEngineError};

pub(crate) async fn send_note_modify(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
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
    store.set_pending_modify(ResourceKind::Note, path, &content_hash);
    store.set_content_hash(
        ResourceKind::Note,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    );
    store.save()?;
    Ok(())
}

pub(crate) async fn send_setting_modify(
    ws: &mut FnsWsClient,
    vault_name: &VaultName,
    vault: &VaultFs,
    store: &mut LocalStore,
    path: &VaultPath,
) -> Result<()> {
    let content = vault.read_text(path)?;
    let metadata = vault.file_metadata(path)?;
    let content_hash = text_content_hash(&content);
    let request = build_setting_modify_request(
        vault_name,
        path,
        content,
        &content_hash,
        metadata.ctime,
        metadata.mtime,
    )?;

    ws.send_json(Action::SettingModify, &request).await?;
    store.set_pending_modify(ResourceKind::Setting, path, &content_hash);
    store.set_content_hash(
        ResourceKind::Setting,
        path,
        Some(content_hash),
        metadata.mtime,
        metadata.size,
    );
    store.save()?;
    Ok(())
}

pub(crate) async fn send_file_upload(
    ws: &mut FnsWsClient,
    vault: &VaultFs,
    store: &mut LocalStore,
    upload: &FileUpload,
    transfer_timeout: Duration,
) -> Result<()> {
    let plan = build_upload_plan(vault, upload)?;
    let start_chunk_index = resume_start_chunk(store, &plan)?;

    for chunk in &plan.chunks {
        if chunk.chunk_index() < start_chunk_index {
            continue;
        }

        send_file_chunk(ws, chunk, transfer_timeout).await?;
        if (chunk.chunk_index() as usize) + 1 < plan.total_chunks {
            store.set_file_upload_checkpoint(
                &plan.path,
                UploadCheckpoint::new(
                    &plan.session_id,
                    plan.content_hash.clone(),
                    chunk.chunk_index(),
                ),
            );
            store.save()?;
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

async fn send_file_chunk(
    ws: &mut FnsWsClient,
    chunk: &fns_protocol::FileChunkFrame,
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

fn resume_start_chunk(store: &LocalStore, plan: &fns_file_transfer::UploadPlan) -> Result<u32> {
    let Some(checkpoint) = store.file_upload_checkpoint(&plan.path) else {
        return Ok(0);
    };

    if checkpoint.session_id != plan.session_id {
        return Ok(0);
    }

    if checkpoint.content_hash()?.as_str() != plan.content_hash.as_str() {
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
