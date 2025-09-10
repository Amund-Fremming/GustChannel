use axum::extract::ws::Utf8Bytes;
use serde::{Deserialize, Serialize};

use crate::error::WsError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub function_name: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Param {
    Type(DataType),
    Value(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    Text(String),
    Number(i32),
}

pub async fn parse_payload(bytes: Utf8Bytes) -> Result<Payload, WsError> {
    let s = bytes.as_str();
    let payload: Payload = serde_json::from_str(s)?;
    Ok(payload)
}
