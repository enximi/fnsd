use fns_core::{RemoteMillis, ResourceKind, VaultPath};
use fns_local_store::LocalStore;
use fns_protocol::{
    FolderDeleteAckMessage, FolderModifyAckMessage, FolderRenameAckMessage,
    FolderSyncDeleteMessage, FolderSyncEndMessage, FolderSyncModifyMessage,
    FolderSyncRenameMessage, TextFrame,
};
use fns_sync_plan::{FolderOperation, plan_folder_delete, plan_folder_modify, plan_folder_rename};
use fns_vault_fs::VaultFs;

use crate::{EventOutcome, Result, local};

pub(crate) fn apply_modify(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: FolderSyncModifyMessage = frame.decode_payload()?;
    let FolderOperation::Create(folder) = plan_folder_modify(&message)? else {
        unreachable!("folder modify planner must produce create operation");
    };
    vault.create_dir_all(&folder.path)?;
    store.set_content_hash(ResourceKind::Folder, &folder.path, None, folder.mtime, 0);
    store.set_sync_time(ResourceKind::Folder, folder.last_time);
    Ok(EventOutcome::RemoteWrite {
        kind: ResourceKind::Folder,
        path: folder.path,
    })
}

pub(crate) fn apply_delete(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: FolderSyncDeleteMessage = frame.decode_payload()?;
    let last_time = RemoteMillis::new(message.last_time)?;
    let FolderOperation::Delete(resource) = plan_folder_delete(&message)? else {
        unreachable!("folder delete planner must produce delete operation");
    };
    local::delete_dir_if_exists(vault, &resource.path)?;
    store.remove_hash_entry(ResourceKind::Folder, &resource.path);
    store.set_sync_time(ResourceKind::Folder, last_time);
    Ok(EventOutcome::RemoteDelete {
        kind: ResourceKind::Folder,
        path: resource.path,
    })
}

pub(crate) fn apply_rename(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: FolderSyncRenameMessage = frame.decode_payload()?;
    let FolderOperation::Rename(rename) = plan_folder_rename(&message)? else {
        unreachable!("folder rename planner must produce rename operation");
    };
    local::rename_path(
        ResourceKind::Folder,
        &rename.old_path,
        &rename.path,
        vault,
        store,
    )?;
    store.set_sync_time(ResourceKind::Folder, rename.last_time);
    Ok(EventOutcome::RemoteRename {
        kind: ResourceKind::Folder,
        old_path: rename.old_path,
        new_path: rename.path,
    })
}

pub(crate) fn sync_end(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FolderSyncEndMessage = frame.decode_payload()?;
    local::sync_end(ResourceKind::Folder, message.last_time, store)
}

pub(crate) fn modify_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FolderModifyAckMessage = frame.decode_payload()?;
    local::ack(
        ResourceKind::Folder,
        &message.path,
        message.last_time,
        store,
    )
}

pub(crate) fn rename_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FolderRenameAckMessage = frame.decode_payload()?;
    let path = VaultPath::new(&message.path)?;
    let last_time = RemoteMillis::new(message.last_time)?;
    local::commit_pending_rename(ResourceKind::Folder, &path, store)?;
    store.set_sync_time(ResourceKind::Folder, last_time);
    Ok(EventOutcome::Ack {
        kind: ResourceKind::Folder,
        path,
    })
}

pub(crate) fn delete_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: FolderDeleteAckMessage = frame.decode_payload()?;
    local::delete_ack(
        ResourceKind::Folder,
        &message.path,
        message.last_time,
        store,
    )
}
