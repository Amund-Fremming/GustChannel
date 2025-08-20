use std::collections::HashMap;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::core::Group;

static BROKER: Lazy<Broker> = Lazy::new(|| Broker::new());

#[derive(Debug)]
pub struct Broker {
    // registry: Mutex<HashMap<String, String>>, // TODO
    groups: RwLock<HashMap<i32, Group>>,
}
impl Broker {
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
        }
    }

    async fn add_to_group(&mut self, key: &i32) {
        let mut lock = self.groups.write().await;
        if let Some(group) = lock.get_mut(key) {
            group
        };
    }

    async fn send_to_group() {
        //
    }
}
