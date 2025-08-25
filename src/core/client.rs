use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use uuid::Uuid;

#[derive(Debug)]
pub struct Client {
    pub id: Uuid,
    pub writer: SplitSink<WebSocket, Message>,
}

impl Client {
    pub fn new(id: Uuid, writer: SplitSink<WebSocket, Message>) -> Self {
        Self { id, writer }
    }
}
