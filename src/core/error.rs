use axum::extract::ws::Message;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("Internal error {0}")]
    Internal(String),

    #[error("Invalid function: `{0}`")]
    InvalidFunction(String),

    #[error("Failed to parse payload: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("JSON parse error: {0}")]
    Json(serde_json::Error),

    #[error("Utf8 parse error: {0}")]
    Utf8(std::str::Utf8Error),

    #[error("Channel error: {0} - {1}")]
    ChannelError(ChannelType, SendError<Message>),

    #[error("Channel closed unexpected")]
    ChannelClosed(ChannelType),
}

#[derive(Debug)]
pub enum ChannelType {
    Group,
    Client,
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Group => write!(f, "Group"),
            ChannelType::Client => write!(f, "Client"),
        }
    }
}
