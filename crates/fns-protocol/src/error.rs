#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("text frame action is empty")]
    EmptyAction,
    #[error("text frame is missing separator")]
    MissingSeparator,
    #[error("unknown action: {0}")]
    UnknownAction(String),
    #[error("response rejected: {0}")]
    ResponseRejected(String),
    #[error("response data is missing")]
    MissingResponseData,
    #[error("binary frame prefix must be exactly 2 bytes")]
    InvalidBinaryPrefix,
    #[error("file chunk frame is shorter than 40 bytes")]
    InvalidFileChunkFrame,
    #[error("file chunk session id must be exactly 36 bytes")]
    InvalidFileChunkSessionId,
    #[error("file chunk session id is not valid utf-8")]
    InvalidFileChunkSessionIdUtf8,
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("protobuf encode error: {0}")]
    ProtobufEncode(#[from] prost::EncodeError),
    #[error("protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("unsupported protobuf action: {0}")]
    UnsupportedProtobufAction(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
