use std::collections::HashMap;

use tokio::sync::{Mutex, RwLock};

use crate::core::{ChannelError, Group, Payload};

#[derive(Debug)]
pub struct Pool {
    handlers: Mutex<HashMap<String, dyn AsyncFn(i32) -> Result<Payload, ChannelError>>>,
    groups: RwLock<HashMap<i32, Group>>,
}
