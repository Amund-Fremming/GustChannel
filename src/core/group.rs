use tokio::sync::{RwLock, broadcast};

use crate::core::{Client, Payload};

#[derive(Debug)]
pub struct Group {
    reciever: broadcast::Receiver<Payload>,
    clients: RwLock<Vec<Client>>,
}
