use axum::extract::ws::Utf8Bytes;
use serde::{Deserialize, Serialize};

use crate::error::WsError;

#[derive(Debug, Serialize, Deserialize)]
pub struct FnRequest {
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
    Str(String),
    Int(i32),
    Bool(bool),
}

pub async fn parse_fn_request(bytes: Utf8Bytes) -> Result<FnRequest, WsError> {
    let s = bytes.as_str();
    let fn_req: FnRequest = serde_json::from_str(s)?;
    Ok(fn_req)
}

pub struct Envelope {
    pub group_id: i32,
    pub json: String,
}
