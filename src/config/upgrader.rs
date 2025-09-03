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

use crate::{broker::Broker, client::Client, config::setup};

pub fn create_websocket_routes<I>(endpoints: I) -> Router
where
    I: IntoIterator<Item = String>,
{
    setup::init_tracing();
    let mut master = Router::new();

    for endpoint in endpoints {
        let fixed_name = format!("/{}/{}", endpoint.trim_start_matches('/'), "{group_id}");
        info!("Creating broker: {}", endpoint.trim_start_matches('/'));
        let slave: Router = Router::new()
            .route(&fixed_name, get(ws_upgrader))
            .with_state(Arc::new(Broker::new(&fixed_name)));

        master = master.merge(slave);
    }

    master
}

pub async fn ws_upgrader(
    State(broker): State<Arc<Broker>>,
    Path(group_id): Path<i32>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(group_id, broker.clone(), socket))
}

pub async fn handle_socket(group_id: i32, broker: Arc<Broker>, socket: WebSocket) {
    let (writer, reader) = socket.split();
    let client_id = Uuid::new_v4();

    let client = Client::new(client_id, writer);
    broker.connect_to_group(group_id, client).await;
    broker.spawn_message_reader(group_id, client_id, reader);
}
