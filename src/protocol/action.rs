use std::fmt;
use std::str::FromStr;

use crate::protocol::ProtocolError;

macro_rules! define_actions {
    ($($variant:ident => $value:literal,)*) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum Action {
            $($variant,)*
        }

        impl Action {
            pub fn as_str(&self) -> &str {
                match self {
                    $(Action::$variant => $value,)*
                }
            }
        }

        impl TryFrom<&str> for Action {
            type Error = ProtocolError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Action::$variant),)*
                    value => Err(ProtocolError::UnknownAction(value.to_string())),
                }
            }
        }
    };
}

define_actions! {
    Authorization => "Authorization",
    ClientInfo => "ClientInfo",

    FolderSync => "FolderSync",
    FolderModify => "FolderModify",
    FolderDelete => "FolderDelete",
    FolderRename => "FolderRename",
    FolderSyncModify => "FolderSyncModify",
    FolderSyncDelete => "FolderSyncDelete",
    FolderSyncEnd => "FolderSyncEnd",
    FolderSyncRename => "FolderSyncRename",
    FolderModifyAck => "FolderModifyAck",
    FolderRenameAck => "FolderRenameAck",
    FolderDeleteAck => "FolderDeleteAck",

    NoteSync => "NoteSync",
    NoteModify => "NoteModify",
    NoteDelete => "NoteDelete",
    NoteRename => "NoteRename",
    NoteCheck => "NoteCheck",
    NoteRePush => "NoteRePush",
    NoteSyncModify => "NoteSyncModify",
    NoteSyncDelete => "NoteSyncDelete",
    NoteSyncRename => "NoteSyncRename",
    NoteSyncMtime => "NoteSyncMtime",
    NoteSyncNeedPush => "NoteSyncNeedPush",
    NoteSyncEnd => "NoteSyncEnd",
    NoteModifyAck => "NoteModifyAck",
    NoteRenameAck => "NoteRenameAck",
    NoteDeleteAck => "NoteDeleteAck",

    FileSync => "FileSync",
    FileUploadCheck => "FileUploadCheck",
    FileDelete => "FileDelete",
    FileRename => "FileRename",
    FileChunkDownload => "FileChunkDownload",
    FileRePush => "FileRePush",
    FileSyncUpdate => "FileSyncUpdate",
    FileSyncDelete => "FileSyncDelete",
    FileSyncRename => "FileSyncRename",
    FileSyncMtime => "FileSyncMtime",
    FileSyncEnd => "FileSyncEnd",
    FileUpload => "FileUpload",
    FileSyncChunkDownload => "FileSyncChunkDownload",
    FileRenameAck => "FileRenameAck",
    FileUploadAck => "FileUploadAck",
    FileDeleteAck => "FileDeleteAck",

    SettingSync => "SettingSync",
    SettingModify => "SettingModify",
    SettingDelete => "SettingDelete",
    SettingCheck => "SettingCheck",
    SettingClear => "SettingClear",
    SettingRePush => "SettingRePush",
    SettingSyncModify => "SettingSyncModify",
    SettingSyncDelete => "SettingSyncDelete",
    SettingSyncMtime => "SettingSyncMtime",
    SettingSyncEnd => "SettingSyncEnd",
    SettingSyncNeedUpload => "SettingSyncNeedUpload",
    SettingSyncClear => "SettingSyncClear",
    SettingModifyAck => "SettingModifyAck",
    SettingDeleteAck => "SettingDeleteAck",
}

impl TryFrom<String> for Action {
    type Error = ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Action::try_from(value.as_str())
    }
}

impl FromStr for Action {
    type Err = ProtocolError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Action::try_from(value)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
