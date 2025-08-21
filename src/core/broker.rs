use std::collections::HashMap;

use axum::extract::ws::WebSocket;
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, RwLock};

use crate::core::Group;

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

    pub async fn add_to_group(&mut self, socket: WebSocket, group_id: i32) {
        let mut lock = self.groups.write().await;
        if let Some(group) = lock.get_mut(&group_id) {
            println!("Adding client to group");
            group.add(socket).await;
            return;
        };

        println!("Creating group");
        lock.insert(group_id, Group::new(socket));
    }
}
