use axum::extract::ws::Utf8Bytes;

use crate::{error::ChannelError, payload::MetaFunction, payload::Payload};

pub async fn parse_payload(bytes: Utf8Bytes) -> Result<Payload, ChannelError> {
    let s = bytes.as_str();
    let payload: Payload = serde_json::from_str(s)?;
    Ok(payload)
}

pub async fn try_execute(payload: Payload, meta_fn: MetaFunction) -> Result<(), ChannelError> {
    let param_len = meta_fn.params.len();
    if payload.params.len() != param_len {
        return Err(ChannelError::InvalidFunction(payload.function_name));
    }

    for _i in 0..param_len {
        //
    }

    Ok(())
}
