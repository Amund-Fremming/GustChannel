use std::net::TcpStream;

use futures_util::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use uuid::Uuid;

use crate::core::Payload;

#[derive(Debug)]
pub struct Client {
    id: Uuid,
    ws_sender: SplitSink<WebSocketStream<TcpStream>, Payload>,
}
