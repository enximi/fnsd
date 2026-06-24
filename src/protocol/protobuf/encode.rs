use prost::Message;
use serde::Serialize;

use crate::protocol::{
    Action, ClientInfoMessage, FileDeleteRequest, FileGetRequest, FileRenameRequest,
    FileSyncRequest, FileUploadCheckRequest, FolderCreateRequest, FolderDeleteRequest,
    FolderRenameRequest, FolderSyncRequest, NoteDeleteRequest, NoteGetRequest,
    NoteModifyOrCreateRequest, NoteRenameRequest, NoteSyncRequest, NoteUpdateCheckRequest,
    ProtocolError, Result, SettingClearRequest, SettingDeleteRequest, SettingGetRequest,
    SettingModifyOrCreateRequest, SettingSyncRequest, SettingUpdateCheckRequest,
};

use super::model::*;

pub fn encode_protobuf_frame<T>(action: Action, payload: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let payload = serde_json::to_value(payload)?;
    let data = encode_request_payload(&action, payload)?;
    Ok(encode_ws_message(action, data))
}

fn encode_ws_message(action: Action, data: Vec<u8>) -> Vec<u8> {
    let envelope = WsMessage {
        message_type: action.as_str().to_string(),
        data,
    };
    let envelope = envelope.encode_to_vec();
    let mut frame = Vec::with_capacity(2 + envelope.len());
    frame.extend_from_slice(PROTOBUF_BINARY_PREFIX.as_bytes());
    frame.extend_from_slice(&envelope);
    frame
}

fn encode_request_payload(action: &Action, payload: serde_json::Value) -> Result<Vec<u8>> {
    macro_rules! encode {
        ($ty:ty, $pb:ty) => {{
            let value: $ty = serde_json::from_value(payload)?;
            Ok(<$pb>::from(value).encode_to_vec())
        }};
    }

    match action {
        Action::ClientInfo => encode!(ClientInfoMessage, PbClientInfoMessage),
        Action::FolderSync => encode!(FolderSyncRequest, PbFolderSyncRequest),
        Action::FolderModify => encode!(FolderCreateRequest, PbFolderCreateRequest),
        Action::FolderDelete => encode!(FolderDeleteRequest, PbFolderDeleteRequest),
        Action::FolderRename => encode!(FolderRenameRequest, PbFolderRenameRequest),
        Action::NoteSync => encode!(NoteSyncRequest, PbNoteSyncRequest),
        Action::NoteModify => encode!(NoteModifyOrCreateRequest, PbNoteModifyOrCreateRequest),
        Action::NoteCheck => encode!(NoteUpdateCheckRequest, PbNoteUpdateCheckRequest),
        Action::NoteDelete => encode!(NoteDeleteRequest, PbNoteDeleteRequest),
        Action::NoteRename => encode!(NoteRenameRequest, PbNoteRenameRequest),
        Action::NoteRePush => encode!(NoteGetRequest, PbNoteGetRequest),
        Action::FileSync => encode!(FileSyncRequest, PbFileSyncRequest),
        Action::FileUploadCheck => encode!(FileUploadCheckRequest, PbFileUploadCheckRequest),
        Action::FileDelete => encode!(FileDeleteRequest, PbFileDeleteRequest),
        Action::FileRename => encode!(FileRenameRequest, PbFileRenameRequest),
        Action::FileChunkDownload => encode!(FileGetRequest, PbFileChunkDownloadRequest),
        Action::FileRePush => encode!(FileGetRequest, PbFileGetRequest),
        Action::SettingSync => encode!(SettingSyncRequest, PbSettingSyncRequest),
        Action::SettingModify => {
            encode!(SettingModifyOrCreateRequest, PbSettingModifyOrCreateRequest)
        }
        Action::SettingCheck => encode!(SettingUpdateCheckRequest, PbSettingUpdateCheckRequest),
        Action::SettingDelete => encode!(SettingDeleteRequest, PbSettingDeleteRequest),
        Action::SettingClear => encode!(SettingClearRequest, PbSettingClearRequest),
        Action::SettingRePush => encode!(SettingGetRequest, PbSettingGetRequest),
        _ => Err(ProtocolError::UnsupportedProtobufAction(action.to_string())),
    }
}
