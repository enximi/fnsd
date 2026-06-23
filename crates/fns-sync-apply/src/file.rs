use fns_core::{RemoteMillis, ResourceKind, VaultPath};
use fns_local_store::LocalStore;
use fns_protocol::{
    FileDeleteAckMessage, FileRenameAckMessage, FileSyncDeleteMessage, FileSyncDownloadMessage,
    FileSyncEndMessage, FileSyncModifyMessage, FileSyncMtimeMessage, FileSyncRenameMessage,
    FileSyncUploadMessage, FileUploadAckMessage, TextFrame,
};
use fns_sync_plan::{
    FileOperation, plan_file_delete, plan_file_download, plan_file_modify, plan_file_mtime,
    plan_file_rename, plan_file_upload,
};
use fns_vault_fs::VaultFs;

use crate::{EventOutcome, Result, local};

pub(crate) fn need_download(frame: &TextFrame) -> Result<EventOutcome> {
    let message: FileSyncModifyMessage = frame.decode_payload()?;
    let FileOperation::Download(file) = plan_file_modify(&message)? else {
        unreachable!("file update planner must produce download operation");
    };
    Ok(EventOutcome::NeedFileDownload(file))
}

pub(crate) fn apply_delete(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: FileSyncDeleteMessage = frame.decode_payload()?;
    let last_time = RemoteMillis::new(message.last_time)?;
    let FileOperation::Delete(resource) = plan_file_delete(&message)? else {
        unreachable!("file delete planner must produce delete operation");
    };
    local::apply_file_delete(ResourceKind::File, resource, last_time, vault, store)
}

pub(crate) fn apply_rename(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: FileSyncRenameMessage = frame.decode_payload()?;
    let FileOperation::Rename(rename) = plan_file_rename(&message)? else {
        unreachable!("file rename planner must produce rename operation");
    };
    local::rename_path(
        ResourceKind::File,
        &rename.old_path,
        &rename.path,
        vault,
        store,
    )?;
    store.set_sync_time(ResourceKind::File, rename.last_time);
    Ok(EventOutcome::RemoteRename {
        kind: ResourceKind::File,
        old_path: rename.old_path,
        new_path: rename.path,
    })
}

pub(crate) fn apply_mtime(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: FileSyncMtimeMessage = frame.decode_payload()?;
    let FileOperation::UpdateMtime(update) = plan_file_mtime(&message)? else {
        unreachable!("file mtime planner must produce mtime operation");
    };
    local::apply_mtime_update(ResourceKind::File, update, vault, store)
}

pub(crate) fn need_upload(frame: &TextFrame) -> Result<EventOutcome> {
    let message: FileSyncUploadMessage = frame.decode_payload()?;
    let FileOperation::Upload(upload) = plan_file_upload(&message)? else {
        unreachable!("file upload planner must produce upload operation");
    };
    Ok(EventOutcome::NeedFileUpload(upload))
}

pub(crate) fn download_session(frame: &TextFrame) -> Result<EventOutcome> {
    let message: FileSyncDownloadMessage = frame.decode_payload()?;
    let FileOperation::ReceiveDownload(download) = plan_file_download(&message)? else {
        unreachable!("file download planner must produce receive-download operation");
    };
    Ok(EventOutcome::NeedFileDownloadSession(download))
}

pub(crate) fn sync_end(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FileSyncEndMessage = frame.decode_payload()?;
    local::sync_end(ResourceKind::File, message.last_time, store)
}

pub(crate) fn upload_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FileUploadAckMessage = frame.decode_payload()?;
    let path = VaultPath::new(&message.path)?;
    let last_time = RemoteMillis::new(message.last_time)?;
    store.remove_pending_modify(ResourceKind::File, &path);
    store.remove_file_upload_checkpoint(&path);
    store.set_sync_time(ResourceKind::File, last_time);
    Ok(EventOutcome::Ack {
        kind: ResourceKind::File,
        path,
    })
}

pub(crate) fn rename_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FileRenameAckMessage = frame.decode_payload()?;
    let path = VaultPath::new(&message.path)?;
    let last_time = RemoteMillis::new(message.last_time)?;
    local::commit_pending_rename(ResourceKind::File, &path, store)?;
    store.set_sync_time(ResourceKind::File, last_time);
    Ok(EventOutcome::Ack {
        kind: ResourceKind::File,
        path,
    })
}

pub(crate) fn delete_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FileDeleteAckMessage = frame.decode_payload()?;
    local::delete_ack(ResourceKind::File, &message.path, message.last_time, store)
}
