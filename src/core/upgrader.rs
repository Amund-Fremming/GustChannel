use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::{StreamExt, stream::SplitStream};
use uuid::Uuid;

use crate::{BROKER, Client};

pub fn ws_routes() -> Router {
    Router::new().route("/", get(ws_upgrader))
}

pub async fn ws_upgrader(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

pub async fn handle_socket(socket: WebSocket) {
    let (writer, reader) = socket.split();
    let client_id = Uuid::new_v4();
    let group_id = 1;

    let client = Client::new(client_id, writer);

    {
        let mut lock = BROKER.lock().await;
        lock.add_to_group(client, 1).await;
    }

    spawn_listener(group_id, client_id, reader);
}

fn spawn_listener(group_id: i32, client_id: Uuid, mut reader: SplitStream<WebSocket>) {
    tokio::task::spawn(async move {
        while let Some(result) = reader.next().await {
            let Ok(message_type) = result else { break };

            match message_type {
                Message::Text(bytes) => {
                    println!("Recieved message from client: {:?}", bytes);
                    let payload: &str = &bytes;
                    let lock = BROKER.lock().await;
                    lock.send_to_group(group_id, payload.to_string()).await;
                    drop(lock);
                }
                Message::Binary(_bytes) => todo!(),
                _ => disconnect_client(group_id, client_id).await,
            }
        }

        disconnect_client(group_id, client_id).await;
    });
}

async fn disconnect_client(group_id: i32, client_id: Uuid) {
    println!("Client disconnected");
    let mut lock = BROKER.lock().await;
    lock.remove_from_group(group_id, client_id).await;
}
