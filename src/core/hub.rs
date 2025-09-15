use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::{StreamExt, stream::SplitStream};
use serde::Serialize;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    client::Client,
    core::group::Group,
    models::Envelope,
    payload::{Payload, parse_payload},
    registry::{Primitive, Registry},
};

static BUFFER_SIZE: usize = 128;

type GroupMap = Arc<RwLock<HashMap<i32, Group>>>;

pub struct Hub {
    pub name: String,
    groups: GroupMap,
    registry: Arc<RwLock<Registry>>,
    channel_writer: mpsc::Sender<Envelope>,
}

impl Hub {
    pub fn new(name: &str) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
        let hub = Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            name: name.to_string(),
            registry: Arc::new(RwLock::new(Registry::new())),
            channel_writer: tx,
        };
        hub.spawn_channel_reader(rx);
        hub
    }

    pub async fn add_registry(&self, registry: Registry) {
        let mut reg = self.registry.write().await;
        *reg = registry;
    }

    pub async fn write_to_channel<T: Serialize>(&self, group_id: i32, data: T) {
        let json = serde_json::to_string(&data).unwrap(); // HANDLE ERROR
        let envelope = Envelope { group_id, json };

        if let Err(error) = self.channel_writer.send(envelope).await {
            // handle error
        }
    }

    fn spawn_channel_reader(&self, mut channel_receiver: mpsc::Receiver<Envelope>) {
        let groups_pointer = self.groups.clone();

        tokio::task::spawn(async move {
            while let Some(envelope) = channel_receiver.recv().await {
                let groups_clone = groups_pointer.clone();
                Self::dispatch_channel_message(groups_clone, envelope).await;
            }

            // Handle error
        });
    }

    async fn dispatch_channel_message(groups: GroupMap, envelope: Envelope) {
        let message = Arc::new(Message::Text(Utf8Bytes::from(envelope.json)));
        let lock = groups.read().await;
        let group_id = envelope.group_id;

        let Some(group) = lock.get(&group_id) else {
            warn!("Client tried to send to non existing group: {}", group_id);
            return;
        };

        let needs_closing = match group.write_to_channel(message).await {
            Ok(_) => false,
            Err(e) => {
                // Close down group
                error!("Failed to add message to group queue: {}", e);
                true
            }
        };

        drop(lock);

        if needs_closing {
            warn!("Purging group and clients");
            let mut lock = groups.write().await;
            if let Some(group) = lock.get_mut(&group_id) {
                group.purge_group_and_clients().await;
            }

            lock.remove(&group_id);
        }
    }

    pub(crate) fn spawn_message_reader(
        &self,
        group_id: i32,
        client_id: Uuid,
        mut reader: SplitStream<WebSocket>,
    ) {
        let endpoint_clone = self.name.clone();
        let groups_pointer = self.groups.clone();
        let registry_pointer = self.registry.clone();

        tokio::task::spawn(async move {
            while let Some(result) = reader.next().await {
                let Ok(message_type) = result else { break };

                match message_type {
                    Message::Text(utf8_bytes) => match parse_payload(utf8_bytes).await {
                        Ok(payload) => {
                            info!("Received a message on broker: {}", endpoint_clone);
                            Self::dispatch_function(group_id, payload, registry_pointer.clone())
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

    pub(crate) async fn connect_to_group(&self, group_id: i32, client: Client) {
        let mut lock = self.groups.write().await;

        let group = match lock.get_mut(&group_id) {
            Some(group) => group,
            None => {
                info!("Creating group: {}", group_id);
                lock.insert(group_id, Group::new(client));
                return;
            }
        };

        info!("Adding client to group: {}", group_id);
        if let Err(e) = group.add_client(client).await {
            error!("Failed to add client to group: {}", e);
            group.purge_group_and_clients().await;
        }
    }

    // Parses json, validates payload, calls function
    async fn dispatch_function(group_id: i32, payload: Payload, registry: Arc<RwLock<Registry>>) {
        debug!(
            "Dispatching function: {}, to group: {}",
            payload.function_name, group_id
        );

        // Optimalization
        // - Get function
        // - Spawn task
        // - Release lock
        // - Call

        let reg = registry.read().await;
        reg.call(
            &payload.function_name,
            vec![Primitive::Text("param".into())],
        )
        .await;
    }

    // Cleanup
    async fn remove_from_group(groups: GroupMap, group_id: i32, client_id: Uuid) {
        let mut lock = groups.write().await;

        let Some(group) = lock.get_mut(&group_id) else {
            return;
        };

        group.purge_client(client_id).await;

        if group.empty().await {
            info!("Group {} is empty, closing down", group_id);
            group.purge_group_and_clients().await;
            lock.remove(&group_id);
        }
    }
}
