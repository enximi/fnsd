use crate::core::ResourceKind;
use crate::store::Result;

mod hash;
mod pending;
mod schema;
mod transfer;

pub(crate) use hash::{
    all_hash_paths, hash_entry, hash_entry_count, hash_tree_paths, remove_hash_entry,
    rename_hash_tree, set_hash_entry, set_sync_time, sync_time,
};
pub(crate) use pending::{
    clear_ack_pending, file_upload_checkpoint, has_pending_modify, insert_pending_delete,
    pending_delete_count, pending_modify_count, pending_rename_count, pop_pending_rename,
    push_pending_rename, remove_file_upload_checkpoint, remove_pending_delete,
    remove_pending_modify, set_file_upload_checkpoint, set_pending_modify,
};
pub(crate) use schema::initialize_schema;
pub(crate) use transfer::{
    clear_download_chunks, download_chunk_count, restore_download_chunks, save_download_chunk,
    upload_checkpoint_count,
};

pub(super) fn all_resource_kinds() -> [ResourceKind; 4] {
    [
        ResourceKind::Note,
        ResourceKind::File,
        ResourceKind::Folder,
        ResourceKind::Setting,
    ]
}

pub(super) fn kind_key(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Note => "note",
        ResourceKind::File => "file",
        ResourceKind::Folder => "folder",
        ResourceKind::Setting => "setting",
    }
}

pub(super) fn to_i64(name: &'static str, value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| crate::store::LocalStoreError::NumberTooLarge { name, value })
}
