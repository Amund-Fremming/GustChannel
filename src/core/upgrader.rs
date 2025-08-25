use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{BROKER, Client};

pub fn ws_routes() -> Router {
    Router::new().route("/", get(ws_upgrader))
}

pub async fn ws_upgrader(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

pub async fn handle_socket(socket: WebSocket) {
    let (writer, mut reader) = socket.split();
    let client_id = Uuid::new_v4();
    let group_id = 1;

    let client = Client::new(client_id, writer);

    {
        let mut lock = BROKER.lock().await;
        lock.add_to_group(client, 1).await;
    }

    tokio::task::spawn(async move {
        while let Some(result) = reader.next().await {
            if let Ok(msg) = result {
                match msg {
                    Message::Text(msg) => {
                        println!("Recieved message from client: {:?}", msg);

                        let payload: &str = &msg;
                        let lock = BROKER.lock().await;
                        lock.send_to_group(group_id, payload.to_string()).await;
                        drop(lock);
                    }
                    Message::Close(_) => {
                        println!("Client disconnected");
                        break;
                    }
                    _ => {
                        println!("Random error occurred");
                        break;
                    }
                }
            } else {
                break;
            }
        }

        {
            let lock = BROKER.lock().await;
            lock.remove_from_group(group_id, client_id).await;
        }
    });
}
