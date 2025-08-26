use std::collections::HashMap;

use once_cell::sync::Lazy;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{Client, core::Group};

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

    pub async fn add_to_group(&mut self, client: Client, group_id: i32) {
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

    pub async fn send_to_group(&self, group_id: i32, payload: String) {
        let lock = self.groups.read().await;
        let Some(group) = lock.get(&group_id) else {
            return;
        };

        group.send_to_group(payload).await;
    }
}
