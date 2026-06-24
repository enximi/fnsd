//! fnsd 的传输协议类型。
//!
//! 该模块只描述协议消息和 frame 编解码。
//! 它不打开 socket，不重试请求，不检查本地文件，也不决定同步策略。

mod action;
mod binary;
mod error;
mod file;
mod folder;
mod frame;
mod handshake;
mod note;
mod protobuf;
mod response;
mod setting;

pub use action::Action;
pub use binary::{
    BINARY_PREFIX_FILE_SYNC, BinaryFrame, FileChunkFrame, decode_binary_frame,
    decode_file_chunk_payload, encode_binary_frame, encode_file_chunk_payload,
};
pub use error::{ProtocolError, Result};
pub use file::{
    FileDeleteAckMessage, FileDeleteRequest, FileGetRequest, FileRenameAckMessage,
    FileRenameRequest, FileSyncCheckRequest, FileSyncDelFile, FileSyncDeleteMessage,
    FileSyncDownloadMessage, FileSyncEndMessage, FileSyncModifyMessage, FileSyncMtimeMessage,
    FileSyncRenameMessage, FileSyncRequest, FileSyncUploadMessage, FileUploadAckMessage,
    FileUploadCheckRequest,
};
pub use folder::{
    FolderCreateRequest, FolderDeleteAckMessage, FolderDeleteRequest, FolderModifyAckMessage,
    FolderRenameAckMessage, FolderRenameRequest, FolderSyncCheckRequest, FolderSyncDelFolder,
    FolderSyncDeleteMessage, FolderSyncEndMessage, FolderSyncModifyMessage,
    FolderSyncRenameMessage, FolderSyncRequest,
};
pub use frame::{TextFrame, decode_text_frame, encode_raw_text_frame, encode_text_frame};
pub use handshake::{ClientInfoMessage, OfflineSyncStrategy};
pub use note::{
    NoteDeleteAckMessage, NoteDeleteRequest, NoteGetRequest, NoteModifyAckMessage,
    NoteModifyOrCreateRequest, NoteRenameAckMessage, NoteRenameRequest, NoteSyncCheckRequest,
    NoteSyncDelNote, NoteSyncDeleteMessage, NoteSyncEndMessage, NoteSyncModifyMessage,
    NoteSyncMtimeMessage, NoteSyncNeedPushMessage, NoteSyncRenameMessage, NoteSyncRequest,
    NoteUpdateCheckRequest,
};
pub use protobuf::{PROTOBUF_BINARY_PREFIX, decode_protobuf_frame, encode_protobuf_frame};
pub use response::WsResponse;
pub use setting::{
    SettingClearRequest, SettingDeleteAckMessage, SettingDeleteRequest, SettingGetRequest,
    SettingModifyAckMessage, SettingModifyOrCreateRequest, SettingSyncCheckRequest,
    SettingSyncDelSetting, SettingSyncDeleteMessage, SettingSyncEndMessage,
    SettingSyncModifyMessage, SettingSyncMtimeMessage, SettingSyncNeedUploadMessage,
    SettingSyncRequest, SettingUpdateCheckRequest,
};
