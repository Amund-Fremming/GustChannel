use std::collections::HashMap;

use axum::extract::ws::{Message, Utf8Bytes};
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{Client, Payload, core::Group};

pub static BROKER: Lazy<Mutex<Broker>> = Lazy::new(|| Mutex::new(Broker::new()));

#[derive(Debug)]
pub struct Broker {
    // registry: Mutex<HashMap<String, String>>,
    groups: RwLock<HashMap<i32, Group>>,
}

impl Broker {
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
        }
    }

    pub async fn connect_to_group(&mut self, client: Client, group_id: i32) {
        let mut groups_lock = self.groups.write().await;

        if let Some(group) = groups_lock.get_mut(&group_id) {
            println!("Adding client to group");
            group.add(client).await;
            return;
        };

        println!("Creating group");
        groups_lock.insert(group_id, Group::new(client));
    }

    pub async fn remove_from_group(&mut self, group_id: i32, client_id: Uuid) {
        let mut lock = self.groups.write().await;

        let Some(group) = lock.get_mut(&group_id) else {
            return;
        };

        group.remove(client_id).await;

        if group.empty().await {
            println!("Group is empty, closing down");
            group.close();
            lock.remove(&group_id);
        }
    }

    // Parses binary, validates payload, calls function
    pub async fn dispatch_function(&self, payload: Payload) {
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());

        // Just fake for now
        self.dispatch_message(1, &"Hello!").await
    }

    // Used in functions to send data to clients after doing some
    pub async fn dispatch_message<T: Serialize>(&self, group_id: i32, data: &T) {
        let json =
            serde_json::to_string(data).expect("HANDLE THIS PLEASE, maybe send a error to user?");
        let message = Message::Text(Utf8Bytes::from(json));

        let lock = self.groups.read().await;
        let Some(group) = lock.get(&group_id) else {
            return;
        };

        group.send_to_group(message).await;
    }
}
