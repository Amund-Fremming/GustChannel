use std::collections::HashSet;

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::{StreamExt, stream::SplitStream};
use uuid::Uuid;

use crate::{
    BROKER, Client,
    core::{parser, payload},
};

pub fn ws_routes() -> Router {
    Router::new().route("/", get(ws_upgrader))
}

// To be used
pub fn create_websocket_routes(endpoints: HashSet<String>) -> Router {
    let mut router = Router::new();

    for endpoint in endpoints {
        let fixed_name = format!("/{}", endpoint.trim_start_matches('/'));
        router = router.route(&fixed_name, get(ws_upgrader));
    }

    router
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
        lock.connect_to_group(client, 1).await;
    }

    spawn_listener(group_id, client_id, reader);
}

fn spawn_listener(group_id: i32, client_id: Uuid, mut reader: SplitStream<WebSocket>) {
    tokio::task::spawn(async move {
        while let Some(result) = reader.next().await {
            let Ok(message_type) = result else { break };

            match message_type {
                Message::Text(utf8_bytes) => match parser::parse_payload(utf8_bytes).await {
                    Ok(payload) => {
                        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                        let lock = BROKER.lock().await;
                        lock.dispatch_function(payload).await;
                    }
                    Err(e) => {
                        println!("Failed to parse payload: {}", e);
                    }
                },
                Message::Close(_) => {
                    println!("Client disconnected");
                    disconnect_client(group_id, client_id).await;
                    return;
                }
                _ => {
                    println!("Recieved data in wrong format");
                    disconnect_client(group_id, client_id).await;
                    return;
                }
            }
        }

        println!("Reader failed");

        disconnect_client(group_id, client_id).await;
    });
}

async fn disconnect_client(group_id: i32, client_id: Uuid) {
    println!("Client disconnected");
    let mut lock = BROKER.lock().await;
    lock.remove_from_group(group_id, client_id).await;
}
