use std::collections::HashMap;

use futures_util::stream::SplitStream;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::WebSocketStream;

use crate::core::{ChannelError, Group, Payload};

#[derive(Debug)]
pub struct Hub {
    registry: Mutex<HashMap<String, dyn Fn(i32) -> Result<Payload, ChannelError>>>,
    groups: RwLock<HashMap<i32, Group>>,
    ws_reciever: SplitStream<WebSocketStream<Payload>>,
}
