use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::{StreamExt, stream::SplitStream};
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    Client, Payload,
    core::{Group, parser},
};

type GroupMap = Arc<RwLock<HashMap<i32, Group>>>;

#[derive(Debug)]
pub struct Broker {
    // registry: Mutex<HashMap<String, String>>,
    pub endpoint: String,
    groups: GroupMap,
}

impl Broker {
    pub fn new(name: &str) -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            endpoint: name.to_string(),
        }
    }

    pub fn spawn_message_reader(
        &self,
        group_id: i32,
        client_id: Uuid,
        mut reader: SplitStream<WebSocket>,
    ) {
        let endpoint_clone = self.endpoint.clone();
        let groups_pointer = self.groups.clone();
        tokio::task::spawn(async move {
            while let Some(result) = reader.next().await {
                let Ok(message_type) = result else { break };

                match message_type {
                    Message::Text(utf8_bytes) => match parser::parse_payload(utf8_bytes).await {
                        Ok(payload) => {
                            info!("Received a message on broker: {}", endpoint_clone);
                            Self::dispatch_function(group_id, payload, groups_pointer.clone())
                                .await;
                        }
                        Err(e) => {
                            error!("Failed to parse payload: {}", e);
                        }
                    },
                    Message::Close(_) => {
                        warn!(
                            "Client disconnected: group_id: {}, client_id: {}",
                            group_id, client_id
                        );
                        Self::remove_from_group(groups_pointer.clone(), group_id, client_id).await;
                        return;
                    }
                    _ => {
                        warn!("Failed to decode incomming data");
                        Self::remove_from_group(groups_pointer.clone(), group_id, client_id).await;
                        return;
                    }
                }
            }

            warn!("Failed to read from connection");
            Self::remove_from_group(groups_pointer.clone(), group_id, client_id).await;
        });
    }

    pub async fn connect_to_group(&self, group_id: i32, client: Client) {
        let mut groups_lock = self.groups.write().await;

        if let Some(group) = groups_lock.get_mut(&group_id) {
            info!("Adding client to group: {}", group_id);
            group.add(client).await;
            return;
        };

        info!("Creating group: {}", group_id);
        groups_lock.insert(group_id, Group::new(client));
    }

    pub async fn remove_from_group(groups: GroupMap, group_id: i32, client_id: Uuid) {
        let mut lock = groups.write().await;

        let Some(group) = lock.get_mut(&group_id) else {
            return;
        };

        group.remove(client_id).await;

        if group.empty().await {
            info!("Group {} is empty, closing down", group_id);
            group.close();
            lock.remove(&group_id);
        }
    }

    // Parses json, validates payload, calls function
    async fn dispatch_function(group_id: i32, payload: Payload, groups: GroupMap) {
        debug!("Dispatching function: {}", payload.function_name);

        // Just fake for now
        Self::dispatch_message(groups, group_id, &"Hello!").await
    }

    // Used in functions to send data to clients after doing some
    async fn dispatch_message<T: Serialize>(groups: GroupMap, group_id: i32, data: &T) {
        let json =
            serde_json::to_string(data).expect("HANDLE THIS PLEASE, maybe send a error to user?");
        let message = Message::Text(Utf8Bytes::from(json));

        let lock = groups.read().await;
        let Some(group) = lock.get(&group_id) else {
            warn!("Client tried to send to non existing group: {}", group_id);
            return;
        };

        let needs_closing = match group.add_to_queue(message).await {
            Ok(_) => false,
            Err(e) => {
                error!("{}", e);
                true
            }
        };

        drop(lock);

        if needs_closing {
            let mut lock = groups.write().await;
            if let Some(group) = lock.get_mut(&group_id) {
                group.close_group_on_channel_failure().await;
            }

            lock.remove(&group_id);
        }
    }
}
