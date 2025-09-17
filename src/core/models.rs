use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum Primitive {
    String(String),
    Number(i32),
    Bool(bool),
}

impl TryFrom<Value> for Primitive {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(Primitive::String(s)),
            Value::Bool(b) => Ok(Primitive::Bool(b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if let Ok(i) = i32::try_from(i) {
                        return Ok(Primitive::Number(i));
                    }
                }

                return Err(format!("Not an integer or out of range: {}", n));
            }
            other => Err(format!("Unsupported type: {:?}", other)),
        }
    }
}

pub fn convert(values: Vec<Value>) -> Result<Vec<Primitive>, String> {
    values.into_iter().map(Primitive::try_from).collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FnRequest {
    pub function_name: String,
    pub params: Vec<serde_json::Value>,
}

pub struct Envelope {
    pub group_id: i32,
    pub json: String,
}
