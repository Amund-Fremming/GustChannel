use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Internal error {0}")]
    Internal(String),
}
