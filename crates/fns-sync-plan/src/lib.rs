//! Pure sync planning helpers.
//!
//! This crate turns local snapshots and remote protocol messages into explicit
//! requests or operations. It does not read files, open sockets, persist state,
//! or execute the operations it returns.

mod error;
mod operation;
mod request;
mod resource;

pub use error::{PlanError, Result};
pub use operation::{
    FileDownload, FileOperation, FileUpload, FolderChange, FolderOperation, MtimeUpdate,
    NoteOperation, RemoteFile, RemoteText, RenameResource, SettingOperation, TextRename,
    plan_file_delete, plan_file_download, plan_file_modify, plan_file_mtime, plan_file_rename,
    plan_file_upload, plan_folder_delete, plan_folder_modify, plan_folder_rename, plan_note_delete,
    plan_note_modify, plan_note_mtime, plan_note_need_push, plan_note_rename, plan_setting_delete,
    plan_setting_modify, plan_setting_mtime, plan_setting_need_upload,
};
pub use request::{
    build_file_sync_request, build_folder_sync_request, build_note_sync_request,
    build_setting_sync_request,
};
pub use resource::{
    DeletedResource, FileResource, FolderResource, NoteResource, SettingResource, SyncBatch,
    TextResource,
};
