use axum::{
    Router,
    extract::{
        Path,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};

use crate::BROKER;

pub fn ws_routes() -> Router {
    Router::new().route("/{group_id}", get(ws_upgrader))
}

pub async fn ws_upgrader(Path(group_id): Path<i32>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, group_id))
}

pub async fn handle_socket(socket: WebSocket, group_id: i32) {
    let mut lock = BROKER.lock().await;
    lock.add_to_group(socket, group_id).await;
}
