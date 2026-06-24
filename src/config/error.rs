pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Core(#[from] crate::core::CoreError),
    #[error(transparent)]
    Source(#[from] config::ConfigError),
    #[error("missing required config field: {0}")]
    MissingField(&'static str),
    #[error("server websocket URL is invalid")]
    InvalidUrl(#[from] url::ParseError),
    #[error("server URL must start with http://, https://, ws://, or wss://")]
    InvalidServerUrl,
    #[error("path config field is empty: {0}")]
    EmptyPath(&'static str),
}
