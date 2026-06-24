use crate::protocol::{BinaryFrame, FileChunkFrame, TextFrame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsEvent {
    Text(TextFrame),
    Binary(BinaryFrame),
    FileChunk(FileChunkFrame),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Closed,
}
