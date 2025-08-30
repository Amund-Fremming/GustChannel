use std::{collections::HashSet, sync::Arc};

use axum::{
    Router,
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::{Broker, Client, core::config};

pub fn create_websocket_routes(endpoints: HashSet<&str>) -> Router {
    config::init_tracing();
    let mut master = Router::new();

    for endpoint in endpoints {
        let fixed_name = format!("/{}", endpoint.trim_start_matches('/'));
        info!("Creating route: {}", fixed_name);
        let slave: Router = Router::new()
            .route(&fixed_name, get(ws_upgrader))
            .with_state(Arc::new(Broker::new(&fixed_name)));

        master = master.merge(slave);
    }

    master
}

pub async fn ws_upgrader(
    State(broker): State<Arc<Broker>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(broker.clone(), socket))
}

pub async fn handle_socket(broker: Arc<Broker>, socket: WebSocket) {
    let (writer, reader) = socket.split();
    let client_id = Uuid::new_v4();
    let group_id = 1;

    let client = Client::new(client_id, writer);

    broker.connect(client).await;
    broker.spawn_message_reader(group_id, client_id, reader);
}
