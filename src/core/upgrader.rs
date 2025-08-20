use axum::{
    Router,
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::StreamExt;

pub fn ws_routes() -> Router {
    Router::new().route("/", get(ws_upgrader))
}

pub async fn ws_upgrader(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

pub async fn handle_socket(mut socket: WebSocket) {
    let (i, x) = socket.split();

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(bytes)) => {
                println!("Recieved message: {}", bytes);
                let text_str: &str = &bytes;
                if text_str == "ping" {
                    socket
                        .send(Message::Text(Utf8Bytes::from_static("pong")))
                        .await
                        .expect("Failed to send message")
                }
            }
            Ok(Message::Binary(bin)) => {
                println!("Recieved binary: {:?}", bin);
            }
            Ok(Message::Close(_)) => {
                println!("Connection was closed by client");
            }
            _ => {
                println!("An error occurred");
            }
        }
    }
}
