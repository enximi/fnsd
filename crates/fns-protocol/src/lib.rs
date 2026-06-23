//! Wire protocol types for the FNS headless client.
//!
//! This crate models protocol messages and frame encoding only. It does not
//! open sockets, retry requests, inspect local files, or decide sync policy.

mod action;
mod binary;
mod error;
mod file;
mod folder;
mod frame;
mod note;
mod response;
mod session;
mod setting;

pub use action::Action;
pub use binary::{
    BINARY_PREFIX_FILE_SYNC, BinaryFrame, FileChunkFrame, decode_binary_frame,
    decode_file_chunk_payload, encode_binary_frame, encode_file_chunk_payload,
};
pub use error::{ProtocolError, Result};
pub use file::{
    FileChunkDownloadRequest, FileDeleteAckMessage, FileDeleteRequest, FileGetRequest,
    FileRenameAckMessage, FileRenameRequest, FileSyncCheckRequest, FileSyncDelFile,
    FileSyncDeleteMessage, FileSyncDownloadMessage, FileSyncEndMessage, FileSyncModifyMessage,
    FileSyncMtimeMessage, FileSyncRenameMessage, FileSyncRequest, FileSyncUploadMessage,
    FileUploadAckMessage, FileUploadCheckRequest,
};
pub use folder::{
    FolderCreateRequest, FolderDeleteAckMessage, FolderDeleteRequest, FolderModifyAckMessage,
    FolderRenameAckMessage, FolderRenameRequest, FolderSyncCheckRequest, FolderSyncDelFolder,
    FolderSyncDeleteMessage, FolderSyncEndMessage, FolderSyncModifyMessage,
    FolderSyncRenameMessage, FolderSyncRequest,
};
pub use frame::{TextFrame, decode_text_frame, encode_text_frame};
pub use note::{
    NoteDeleteAckMessage, NoteDeleteRequest, NoteGetRequest, NoteModifyAckMessage,
    NoteModifyOrCreateRequest, NoteRenameAckMessage, NoteRenameRequest, NoteSyncCheckRequest,
    NoteSyncDelNote, NoteSyncDeleteMessage, NoteSyncEndMessage, NoteSyncModifyMessage,
    NoteSyncMtimeMessage, NoteSyncNeedPushMessage, NoteSyncRenameMessage, NoteSyncRequest,
    NoteUpdateCheckRequest,
};
pub use response::WsResponse;
pub use session::{AuthorizationRequest, ClientInfoMessage, OfflineSyncStrategy};
pub use setting::{
    SettingClearRequest, SettingDeleteAckMessage, SettingDeleteRequest, SettingGetRequest,
    SettingModifyAckMessage, SettingModifyOrCreateRequest, SettingSyncCheckRequest,
    SettingSyncDelSetting, SettingSyncDeleteMessage, SettingSyncEndMessage,
    SettingSyncModifyMessage, SettingSyncMtimeMessage, SettingSyncNeedUploadMessage,
    SettingSyncRequest, SettingUpdateCheckRequest,
};
