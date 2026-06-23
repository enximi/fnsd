use fns_core::{RemoteMillis, ResourceKind, VaultPath};
use fns_local_store::LocalStore;
use fns_protocol::{
    NoteDeleteAckMessage, NoteModifyAckMessage, NoteRenameAckMessage, NoteSyncDeleteMessage,
    NoteSyncEndMessage, NoteSyncModifyMessage, NoteSyncMtimeMessage, NoteSyncNeedPushMessage,
    NoteSyncRenameMessage, TextFrame,
};
use fns_sync_plan::{
    NoteOperation, plan_note_delete, plan_note_modify, plan_note_mtime, plan_note_need_push,
    plan_note_rename,
};
use fns_vault_fs::VaultFs;

use crate::{EventOutcome, Result, local, pending_sync_end_events};

pub(crate) fn apply_modify(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: NoteSyncModifyMessage = frame.decode_response_data()?;
    let NoteOperation::Write(text) = plan_note_modify(&message)? else {
        unreachable!("note modify planner must produce write operation");
    };
    local::apply_remote_text(ResourceKind::Note, text, vault, store)
}

pub(crate) fn apply_delete(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: NoteSyncDeleteMessage = frame.decode_response_data()?;
    let last_time = RemoteMillis::new(message.last_time)?;
    let NoteOperation::Delete(resource) = plan_note_delete(&message)? else {
        unreachable!("note delete planner must produce delete operation");
    };
    local::apply_file_delete(ResourceKind::Note, resource, last_time, vault, store)
}

pub(crate) fn apply_rename(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: NoteSyncRenameMessage = frame.decode_response_data()?;
    let NoteOperation::Rename(rename) = plan_note_rename(&message)? else {
        unreachable!("note rename planner must produce rename operation");
    };
    local::apply_text_rename(ResourceKind::Note, rename, vault, store)
}

pub(crate) fn apply_mtime(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    let message: NoteSyncMtimeMessage = frame.decode_response_data()?;
    let NoteOperation::UpdateMtime(update) = plan_note_mtime(&message)? else {
        unreachable!("note mtime planner must produce mtime operation");
    };
    local::apply_mtime_update(ResourceKind::Note, update, vault, store)
}

pub(crate) fn need_push(frame: &TextFrame) -> Result<EventOutcome> {
    let message: NoteSyncNeedPushMessage = frame.decode_response_data()?;
    let NoteOperation::Upload(resource) = plan_note_need_push(&message)? else {
        unreachable!("note need-push planner must produce upload operation");
    };
    Ok(EventOutcome::NeedNoteUpload(resource))
}

pub(crate) fn sync_end(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: NoteSyncEndMessage = frame.decode_response_data()?;
    local::sync_end(
        ResourceKind::Note,
        message.last_time,
        pending_sync_end_events(
            message.need_upload_count,
            message.need_modify_count,
            message.need_sync_mtime_count,
            message.need_delete_count,
        ),
        store,
    )
}

pub(crate) fn modify_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: NoteModifyAckMessage = frame.decode_response_data()?;
    let path = VaultPath::new(&message.path)?;
    let last_time = RemoteMillis::new(message.last_time)?;
    store.remove_pending_modify(ResourceKind::Note, &path);
    store.set_sync_time(ResourceKind::Note, last_time);
    Ok(EventOutcome::Ack {
        kind: ResourceKind::Note,
        path,
    })
}

pub(crate) fn rename_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: NoteRenameAckMessage = frame.decode_response_data()?;
    let path = VaultPath::new(&message.path)?;
    let last_time = RemoteMillis::new(message.last_time)?;
    local::commit_pending_rename(ResourceKind::Note, &path, store)?;
    store.set_sync_time(ResourceKind::Note, last_time);
    Ok(EventOutcome::Ack {
        kind: ResourceKind::Note,
        path,
    })
}

pub(crate) fn delete_ack(frame: &TextFrame, store: &mut LocalStore) -> Result<EventOutcome> {
    let message: NoteDeleteAckMessage = frame.decode_response_data()?;
    local::delete_ack(ResourceKind::Note, &message.path, message.last_time, store)
}
