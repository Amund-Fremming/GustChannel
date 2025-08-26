use axum::body::Bytes;

use crate::{ChannelError, MetaFunction, Payload};

pub async fn parse_payload(bytes: Bytes) -> Result<Payload, ChannelError> {
    let payload: Payload = serde_json::from_slice(&bytes)?;
    Ok(payload)
}

pub async fn try_execute(payload: Payload, meta_fn: MetaFunction) -> Result<(), ChannelError> {
    let param_len = meta_fn.params.len();
    if payload.params.len() != param_len {
        return Err(ChannelError::InvalidFunction(payload.function));
    }

    for i in 0..param_len {
        //
    }

    Ok(())
}
