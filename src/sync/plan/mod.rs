//! 纯同步规划辅助函数。
//!
//! 该模块把本地快照和远程协议消息转换成明确的请求或操作。
//! 它不读取文件，不打开 socket，不持久化状态，也不执行返回的操作。

mod error;
mod remote_event;
mod request;

pub use error::{PlanError, Result};
pub use remote_event::{
    FileDownload, FileOperation, FileUpload, FolderOperation, MtimeUpdate, NoteOperation,
    RemoteFile, RemoteText, RemoteTextRename, SettingOperation, plan_file_delete,
    plan_file_download, plan_file_modify, plan_file_mtime, plan_file_rename, plan_file_upload,
    plan_folder_delete, plan_folder_modify, plan_folder_rename, plan_note_delete, plan_note_modify,
    plan_note_mtime, plan_note_need_push, plan_note_rename, plan_setting_delete,
    plan_setting_modify, plan_setting_mtime, plan_setting_need_upload,
};
pub use request::{
    build_file_sync_request, build_folder_sync_request, build_note_modify_request,
    build_note_sync_request, build_setting_modify_request, build_setting_sync_request,
};
