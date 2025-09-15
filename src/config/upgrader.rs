use std::sync::Arc;

use axum::{
    Router,
    extract::{
        Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::{client::Client, config::setup, hub::Hub};

pub fn create_hubs<I>(hubs: I) -> Router
where
    I: IntoIterator<Item = Arc<Hub>>,
{
    setup::init_tracing();
    let mut master = Router::new();

    for hub in hubs {
        let fixed_name = format!("/{}/{}", hub.name.trim_start_matches('/'), "{group_id}");
        info!("Creating broker: {}", hub.name);

        let slave: Router = Router::new()
            .route(&fixed_name, get(ws_upgrader))
            .with_state(hub);

        master = master.merge(slave);
    }

    master
}

pub async fn ws_upgrader(
    State(hub): State<Arc<Hub>>,
    Path(group_id): Path<i32>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(group_id, hub.clone(), socket))
}

pub async fn handle_socket(group_id: i32, hub: Arc<Hub>, socket: WebSocket) {
    let (writer, reader) = socket.split();
    let client_id = Uuid::new_v4();

    let client = Client::new(client_id, writer);
    hub.connect_to_group(group_id, client).await;
    hub.spawn_message_reader(group_id, client_id, reader);
}
