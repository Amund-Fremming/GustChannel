use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::extract::ws::Message;
use tokio::{
    sync::{
        Mutex,
        mpsc::{self},
    },
    task::JoinHandle,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    client::Client,
    error::{ChannelError, ChannelType},
};

static BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Group {
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
    channel_writer: mpsc::Sender<Message>,
    group_writer_task: Option<JoinHandle<bool>>,
}

impl Group {
    pub fn new(client: Client) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);

        let mut group = Self {
            clients: Arc::new(Mutex::new(HashMap::from([(client.id, client)]))),
            channel_writer: tx,
            group_writer_task: None,
        };
        group.spawn_broadcaster(rx);
        group
    }

    pub async fn empty(&self) -> bool {
        let lock = self.clients.lock().await;
        lock.len() == 0
    }

    fn spawn_broadcaster(&mut self, mut receiver: mpsc::Receiver<Message>) {
        let clients_pointer = self.clients.clone();

        self.group_writer_task = Some(tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let mut lock = clients_pointer.lock().await;

                let mut failed_keys = HashSet::new();
                for (k, v) in lock.iter_mut() {
                    if let Err(e) = v.add_to_queue(message.clone()).await {
                        error!("{}", e);
                        failed_keys.insert(k.clone());
                    }
                }

                lock.retain(|k, _v| !failed_keys.contains(k));
            }

            error!("Group channel was closed unexpected");
            return false;
        }));
    }

    pub async fn add_client(&mut self, client: Client) {
        let mut lock = self.clients.lock().await;
        lock.insert(client.id, client);
    }

    pub async fn add_to_queue(&self, message: Message) -> Result<(), ChannelError> {
        let writer_clone = self.channel_writer.clone();

        if let Err(e) = writer_clone.send(message).await {
            error!("Group channel is down, error: {}", e);
            return Err(ChannelError::ChannelError(ChannelType::Group, e));
        };

        Ok(())
    }

    /* Cleanup */

    pub async fn purge_client(&mut self, client_id: Uuid) {
        let mut lock = self.clients.lock().await;
        if let Some(client) = lock.get_mut(&client_id) {
            client.purge();
            lock.retain(|k, _v| *k != client_id);
        }
    }

    pub async fn purge_group_and_clients(&mut self) {
        let mut lock = self.clients.lock().await;
        for (_k, client) in lock.iter_mut() {
            client.purge();
        }

        lock.clear();

        if let Some(task) = self.group_writer_task.take() {
            task.abort();
        };
    }
}
