pub type Result<T> = std::result::Result<T, WsClientError>;

#[derive(Debug, thiserror::Error)]
pub enum WsClientError {
    #[error(transparent)]
    Protocol(#[from] crate::protocol::ProtocolError),
    #[error(transparent)]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("websocket connection closed")]
    Closed,
}
