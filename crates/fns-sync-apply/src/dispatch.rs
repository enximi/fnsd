use fns_local_store::LocalStore;
use fns_protocol::{Action, TextFrame, WsResponse};
use fns_vault_fs::VaultFs;

use crate::{EventOutcome, Result, SyncApplyError, file, folder, note, setting};

pub fn apply_text_event(
    frame: &TextFrame,
    vault: &VaultFs,
    store: &mut LocalStore,
) -> Result<EventOutcome> {
    match frame.action() {
        Action::Authorization => apply_authorization(frame),

        Action::NoteSyncModify => note::apply_modify(frame, vault, store),
        Action::NoteSyncDelete => note::apply_delete(frame, vault, store),
        Action::NoteSyncRename => note::apply_rename(frame, vault, store),
        Action::NoteSyncMtime => note::apply_mtime(frame, vault, store),
        Action::NoteSyncNeedPush => note::need_push(frame),
        Action::NoteSyncEnd => note::sync_end(frame, store),
        Action::NoteModifyAck => note::modify_ack(frame, store),
        Action::NoteRenameAck => note::rename_ack(frame, store),
        Action::NoteDeleteAck => note::delete_ack(frame, store),

        Action::FileSyncUpdate => file::need_download(frame),
        Action::FileSyncDelete => file::apply_delete(frame, vault, store),
        Action::FileSyncRename => file::apply_rename(frame, vault, store),
        Action::FileSyncMtime => file::apply_mtime(frame, vault, store),
        Action::FileUpload => file::need_upload(frame),
        Action::FileSyncChunkDownload => file::download_session(frame),
        Action::FileSyncEnd => file::sync_end(frame, store),
        Action::FileUploadAck => file::upload_ack(frame, store),
        Action::FileRenameAck => file::rename_ack(frame, store),
        Action::FileDeleteAck => file::delete_ack(frame, store),

        Action::FolderSyncModify => folder::apply_modify(frame, vault, store),
        Action::FolderSyncDelete => folder::apply_delete(frame, vault, store),
        Action::FolderSyncRename => folder::apply_rename(frame, vault, store),
        Action::FolderSyncEnd => folder::sync_end(frame, store),
        Action::FolderModifyAck => folder::modify_ack(frame, store),
        Action::FolderRenameAck => folder::rename_ack(frame, store),
        Action::FolderDeleteAck => folder::delete_ack(frame, store),

        Action::SettingSyncModify => setting::apply_modify(frame, vault, store),
        Action::SettingSyncDelete => setting::apply_delete(frame, vault, store),
        Action::SettingSyncMtime => setting::apply_mtime(frame, vault, store),
        Action::SettingSyncNeedUpload => setting::need_upload(frame),
        Action::SettingSyncEnd => setting::sync_end(frame, store),
        Action::SettingModifyAck => setting::modify_ack(frame, store),
        Action::SettingDeleteAck => setting::delete_ack(frame, store),

        _ => Ok(EventOutcome::Ignored),
    }
}

fn apply_authorization(frame: &TextFrame) -> Result<EventOutcome> {
    let response: WsResponse = frame.decode_payload()?;

    if !response.status {
        return Err(SyncApplyError::AuthorizationRejected(response.message));
    }

    Ok(EventOutcome::AuthorizationAccepted)
}
