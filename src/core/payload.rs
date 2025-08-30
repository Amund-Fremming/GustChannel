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

// TODO - delete
#[derive(Debug, Serialize, Deserialize)]
pub struct TestData {
    name: String,
    active: bool,
    iteration: i32,
    ticks: Vec<bool>,
}

impl TestData {
    pub fn new() -> Self {
        Self {
            name: "test-data".into(),
            active: true,
            iteration: 700,
            ticks: vec![true, true, false, true],
        }
    }
}
