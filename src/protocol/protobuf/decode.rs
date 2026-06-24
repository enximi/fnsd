use prost::Message;

use crate::protocol::{Action, ProtocolError, Result, TextFrame, WsResponse};

use super::model::*;

pub fn decode_protobuf_frame(frame: &[u8]) -> Result<TextFrame> {
    let envelope = WsMessage::decode(frame)?;
    let action = Action::try_from(envelope.message_type)?;
    let response = WsResponseMessage::decode(envelope.data.as_slice())?;
    let data = decode_response_payload(&action, &response.data)?;

    let response = WsResponse {
        code: response.code,
        status: response.status,
        message: response.message,
        data,
        details: optional_string(response.details),
        vault: optional_string(response.vault),
        context: optional_string(response.context),
    };
    Ok(TextFrame::new(action, serde_json::to_string(&response)?))
}

fn decode_response_payload(action: &Action, data: &[u8]) -> Result<Option<serde_json::Value>> {
    if data.is_empty() {
        return Ok(None);
    }

    macro_rules! decode {
        ($ty:ty) => {{
            let decoded = <$ty>::decode(data)?;
            Some(serde_json::to_value(decoded)?)
        }};
    }

    let data = match action {
        Action::ClientInfo => decode!(CheckVersionInfo),
        Action::NoteSyncModify => decode!(PbNoteSyncModifyMessage),
        Action::NoteSyncDelete => decode!(PbNoteSyncDeleteMessage),
        Action::NoteSyncRename => decode!(PbNoteSyncRenameMessage),
        Action::NoteSyncMtime => decode!(PbNoteSyncMtimeMessage),
        Action::NoteSyncEnd => decode!(PbNoteSyncEndMessage),
        Action::NoteSyncNeedPush => decode!(PbNoteSyncNeedPushMessage),
        Action::NoteModifyAck | Action::NoteRenameAck | Action::NoteDeleteAck => {
            decode!(PbAckMessage)
        }
        Action::FileSyncUpdate => decode!(PbFileSyncModifyMessage),
        Action::FileSyncDelete => decode!(PbFileSyncDeleteMessage),
        Action::FileSyncRename => decode!(PbFileSyncRenameMessage),
        Action::FileSyncMtime => decode!(PbFileSyncMtimeMessage),
        Action::FileSyncEnd => decode!(PbNoteSyncEndMessage),
        Action::FileUpload => decode!(PbFileSyncUploadMessage),
        Action::FileSyncChunkDownload => decode!(PbFileSyncDownloadMessage),
        Action::FileUploadAck | Action::FileRenameAck | Action::FileDeleteAck => {
            decode!(PbAckMessage)
        }
        Action::SettingSyncModify => decode!(PbSettingSyncModifyMessage),
        Action::SettingSyncDelete => decode!(PbSettingSyncDeleteMessage),
        Action::SettingSyncMtime => decode!(PbSettingSyncMtimeMessage),
        Action::SettingSyncEnd => decode!(PbNoteSyncEndMessage),
        Action::SettingSyncNeedUpload => decode!(PbSettingSyncNeedUploadMessage),
        Action::SettingModifyAck | Action::SettingDeleteAck => decode!(PbAckMessage),
        Action::SettingSyncClear => None,
        Action::FolderSyncModify => decode!(PbFolderSyncModifyMessage),
        Action::FolderSyncDelete => decode!(PbFolderSyncDeleteMessage),
        Action::FolderSyncRename => decode!(PbFolderSyncRenameMessage),
        Action::FolderSyncEnd => decode!(PbFolderSyncEndMessage),
        Action::FolderModifyAck | Action::FolderRenameAck | Action::FolderDeleteAck => {
            decode!(PbAckMessage)
        }
        _ => return Err(ProtocolError::UnsupportedProtobufAction(action.to_string())),
    };

    Ok(data)
}

fn optional_string(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}
