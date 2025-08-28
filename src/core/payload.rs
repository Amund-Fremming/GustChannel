use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub function_name: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug)]
pub struct MetaFunction {
    pub signature: String,
    pub params: Vec<Param>,
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
