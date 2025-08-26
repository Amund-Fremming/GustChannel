use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Internal error {0}")]
    Internal(String),

    #[error("Invalid function: `{0}`")]
    InvalidFunction(String),

    #[error("Failed to parse payload: {0}")]
    ParseError(#[from] serde_json::Error),
}
