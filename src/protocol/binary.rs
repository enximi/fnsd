use crate::protocol::{ProtocolError, Result};

pub const BINARY_PREFIX_FILE_SYNC: &str = "00";
pub const FILE_CHUNK_HEADER_LEN: usize = 40;
pub const FILE_CHUNK_SESSION_ID_LEN: usize = 36;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryFrame {
    prefix: [u8; 2],
    payload: Vec<u8>,
}

impl BinaryFrame {
    pub fn new(prefix: &str, payload: impl Into<Vec<u8>>) -> Result<Self> {
        let prefix = parse_prefix(prefix)?;

        Ok(Self {
            prefix,
            payload: payload.into(),
        })
    }

    pub fn prefix(&self) -> [u8; 2] {
        self.prefix
    }

    pub fn prefix_str(&self) -> String {
        String::from_utf8_lossy(&self.prefix).into_owned()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_parts(self) -> ([u8; 2], Vec<u8>) {
        (self.prefix, self.payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChunkFrame {
    session_id: String,
    chunk_index: u32,
    chunk_data: Vec<u8>,
}

impl FileChunkFrame {
    pub fn new(
        session_id: impl Into<String>,
        chunk_index: u32,
        chunk_data: impl Into<Vec<u8>>,
    ) -> Result<Self> {
        let session_id = session_id.into();
        if session_id.as_bytes().len() != FILE_CHUNK_SESSION_ID_LEN {
            return Err(ProtocolError::InvalidFileChunkSessionId);
        }

        Ok(Self {
            session_id,
            chunk_index,
            chunk_data: chunk_data.into(),
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn chunk_index(&self) -> u32 {
        self.chunk_index
    }

    pub fn chunk_data(&self) -> &[u8] {
        &self.chunk_data
    }

    pub fn into_parts(self) -> (String, u32, Vec<u8>) {
        (self.session_id, self.chunk_index, self.chunk_data)
    }
}

pub fn encode_binary_frame(prefix: &str, payload: &[u8]) -> Result<Vec<u8>> {
    let prefix = parse_prefix(prefix)?;
    let mut frame = Vec::with_capacity(2 + payload.len());
    frame.extend_from_slice(&prefix);
    frame.extend_from_slice(payload);
    Ok(frame)
}

pub fn decode_binary_frame(frame: &[u8]) -> Result<BinaryFrame> {
    if frame.len() < 2 {
        return Err(ProtocolError::InvalidBinaryPrefix);
    }

    Ok(BinaryFrame {
        prefix: [frame[0], frame[1]],
        payload: frame[2..].to_vec(),
    })
}

pub fn encode_file_chunk_payload(chunk: &FileChunkFrame) -> Vec<u8> {
    let mut payload = Vec::with_capacity(FILE_CHUNK_HEADER_LEN + chunk.chunk_data.len());
    payload.extend_from_slice(chunk.session_id.as_bytes());
    payload.extend_from_slice(&chunk.chunk_index.to_be_bytes());
    payload.extend_from_slice(&chunk.chunk_data);
    payload
}

pub fn decode_file_chunk_payload(payload: &[u8]) -> Result<FileChunkFrame> {
    if payload.len() < FILE_CHUNK_HEADER_LEN {
        return Err(ProtocolError::InvalidFileChunkFrame);
    }

    let session_id = std::str::from_utf8(&payload[..FILE_CHUNK_SESSION_ID_LEN])
        .map_err(|_| ProtocolError::InvalidFileChunkSessionIdUtf8)?
        .to_string();
    let chunk_index = u32::from_be_bytes([payload[36], payload[37], payload[38], payload[39]]);
    let chunk_data = payload[FILE_CHUNK_HEADER_LEN..].to_vec();

    FileChunkFrame::new(session_id, chunk_index, chunk_data)
}

fn parse_prefix(prefix: &str) -> Result<[u8; 2]> {
    let bytes = prefix.as_bytes();
    if bytes.len() != 2 {
        return Err(ProtocolError::InvalidBinaryPrefix);
    }

    Ok([bytes[0], bytes[1]])
}
