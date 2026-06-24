use crate::core::{
    FileResource, FolderResource, NoteResource, ResourceKind, SettingResource, SyncBatch,
};
use crate::store::LocalStore;

pub fn refresh_sent_hash_index(
    store: &mut LocalStore,
    notes: &SyncBatch<NoteResource>,
    files: &SyncBatch<FileResource>,
    folders: &SyncBatch<FolderResource>,
    settings: &SyncBatch<SettingResource>,
) {
    for item in &notes.items {
        store.set_content_hash(
            ResourceKind::Note,
            &item.path,
            Some(item.content_hash.clone()),
            item.mtime,
            0,
        );
    }

    for item in &files.items {
        store.set_content_hash(
            ResourceKind::File,
            &item.path,
            Some(item.content_hash.clone()),
            item.mtime,
            item.size,
        );
    }

    for item in &folders.items {
        store.set_content_hash(ResourceKind::Folder, &item.path, None, item.mtime, 0);
    }

    for item in &settings.items {
        store.set_content_hash(
            ResourceKind::Setting,
            &item.path,
            Some(item.content_hash.clone()),
            item.mtime,
            0,
        );
    }

    for item in &notes.deleted {
        store.insert_pending_delete(ResourceKind::Note, &item.path);
    }

    for item in &files.deleted {
        store.insert_pending_delete(ResourceKind::File, &item.path);
    }

    for item in &folders.deleted {
        store.insert_pending_delete(ResourceKind::Folder, &item.path);
    }

    for item in &settings.deleted {
        store.insert_pending_delete(ResourceKind::Setting, &item.path);
    }
}
