use fns_protocol::{
    Action, BINARY_PREFIX_FILE_SYNC, ClientInfoMessage, FileChunkFrame, decode_binary_frame,
    decode_file_chunk_payload, decode_protobuf_frame, decode_text_frame, encode_binary_frame,
    encode_file_chunk_payload, encode_protobuf_client_info, encode_protobuf_frame,
    encode_raw_text_frame, encode_text_frame,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Bytes, Message},
};

use crate::{Result, WsEvent};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct FnsWsClient {
    stream: WsStream,
    protobuf_enabled: bool,
}

impl FnsWsClient {
    pub async fn connect(url: &str) -> Result<Self> {
        let (stream, _) = connect_async(url).await?;
        Ok(Self {
            stream,
            protobuf_enabled: false,
        })
    }

    pub async fn authorize(&mut self, token: impl Into<String>) -> Result<()> {
        self.send_raw_text(Action::Authorization, &token.into())
            .await
    }

    pub async fn send_client_info(&mut self, info: &ClientInfo) -> Result<()> {
        self.send_json(Action::ClientInfo, &info.to_protocol())
            .await
    }

    pub async fn send_protobuf_client_info(&mut self, info: &ClientInfo) -> Result<()> {
        let frame = encode_protobuf_client_info(&info.to_protocol())?;
        self.stream.send(Message::Binary(frame.into())).await?;
        Ok(())
    }

    pub fn enable_protobuf(&mut self) {
        self.protobuf_enabled = true;
    }

    pub fn protobuf_enabled(&self) -> bool {
        self.protobuf_enabled
    }

    pub async fn send_json<T>(&mut self, action: Action, payload: &T) -> Result<()>
    where
        T: Serialize,
    {
        if self.protobuf_enabled {
            let frame = encode_protobuf_frame(action, payload)?;
            self.stream.send(Message::Binary(frame.into())).await?;
        } else {
            let frame = encode_text_frame(action, payload)?;
            self.stream.send(Message::Text(frame.into())).await?;
        }
        Ok(())
    }

    pub async fn send_raw_text(&mut self, action: Action, payload: &str) -> Result<()> {
        let frame = encode_raw_text_frame(action, payload);
        self.stream.send(Message::Text(frame.into())).await?;
        Ok(())
    }

    pub async fn send_file_chunk(&mut self, chunk: &FileChunkFrame) -> Result<()> {
        let payload = encode_file_chunk_payload(chunk);
        let frame = encode_binary_frame(BINARY_PREFIX_FILE_SYNC, &payload)?;
        self.stream.send(Message::Binary(frame.into())).await?;
        Ok(())
    }

    pub async fn send_binary(&mut self, prefix: &str, payload: &[u8]) -> Result<()> {
        let frame = encode_binary_frame(prefix, payload)?;
        self.stream.send(Message::Binary(frame.into())).await?;
        Ok(())
    }

    pub async fn next_event(&mut self) -> Result<WsEvent> {
        loop {
            let Some(message) = self.stream.next().await else {
                return Ok(WsEvent::Closed);
            };

            match message? {
                Message::Text(text) => return Ok(WsEvent::Text(decode_text_frame(&text)?)),
                Message::Binary(bytes) => return self.decode_binary_event(&bytes),
                Message::Ping(bytes) => return Ok(WsEvent::Ping(bytes.to_vec())),
                Message::Pong(bytes) => return Ok(WsEvent::Pong(bytes.to_vec())),
                Message::Close(_) => return Ok(WsEvent::Closed),
                Message::Frame(_) => {}
            }
        }
    }

    pub async fn close(mut self) -> Result<()> {
        self.stream.close(None).await?;
        Ok(())
    }

    fn decode_binary_event(&self, bytes: &Bytes) -> Result<WsEvent> {
        let frame = decode_binary_frame(bytes)?;

        if frame.prefix_str() == BINARY_PREFIX_FILE_SYNC {
            let chunk = decode_file_chunk_payload(frame.payload())?;
            return Ok(WsEvent::FileChunk(chunk));
        }

        if frame.prefix_str() == fns_protocol::PROTOBUF_BINARY_PREFIX {
            return Ok(WsEvent::Text(decode_protobuf_frame(frame.payload())?));
        }

        Ok(WsEvent::Binary(frame))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub client_type: String,
    pub is_desktop: bool,
    pub is_mobile: bool,
    pub is_phone: bool,
    pub is_tablet: bool,
    pub is_mac_os: bool,
    pub is_win: bool,
    pub is_linux: bool,
    pub offline_sync_strategy: Option<fns_protocol::OfflineSyncStrategy>,
    pub protobuf: bool,
}

impl ClientInfo {
    pub fn headless(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            client_type: "headless".to_string(),
            is_desktop: false,
            is_mobile: false,
            is_phone: false,
            is_tablet: false,
            is_mac_os: cfg!(target_os = "macos"),
            is_win: cfg!(target_os = "windows"),
            is_linux: cfg!(target_os = "linux"),
            offline_sync_strategy: None,
            protobuf: false,
        }
    }

    fn to_protocol(&self) -> ClientInfoMessage {
        ClientInfoMessage {
            name: self.name.clone(),
            version: self.version.clone(),
            client_type: self.client_type.clone(),
            is_desktop: self.is_desktop,
            is_mobile: self.is_mobile,
            is_phone: self.is_phone,
            is_tablet: self.is_tablet,
            is_mac_os: self.is_mac_os,
            is_win: self.is_win,
            is_linux: self.is_linux,
            offline_sync_strategy: self.offline_sync_strategy,
            protobuf: self.protobuf,
        }
    }
}
