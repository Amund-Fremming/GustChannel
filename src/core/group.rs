use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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
    error::{ChannelType, WsError},
};

static BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Group {
    clients: Arc<Mutex<HashMap<Uuid, Client>>>,
    channel_writer: mpsc::Sender<Arc<Message>>,
    group_writer_task: Option<JoinHandle<()>>,
    active_flag: Arc<AtomicBool>,
}

impl Group {
    pub fn new(client: Client) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);

        let mut group = Self {
            clients: Arc::new(Mutex::new(HashMap::from([(client.id, client)]))),
            channel_writer: tx,
            group_writer_task: None,
            active_flag: Arc::new(AtomicBool::new(true)),
        };
        group.spawn_broadcaster(rx);
        group
    }

    pub async fn empty(&self) -> bool {
        let lock = self.clients.lock().await;
        lock.len() == 0
    }

    fn spawn_broadcaster(&mut self, mut receiver: mpsc::Receiver<Arc<Message>>) {
        let clients_pointer = self.clients.clone();
        let flag_pointer = self.active_flag.clone();

        self.group_writer_task = Some(tokio::task::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let mut failed_keys = HashSet::new();

                let mut lock = clients_pointer.lock().await;
                for (id, client) in lock.iter_mut() {
                    if let Err(e) = client.add_to_queue(message.clone()).await {
                        error!("Failed to add message to client queue: {}", e);
                        failed_keys.insert(*id);
                    }
                }

                if !failed_keys.is_empty() {
                    lock.retain(|id, _client| !failed_keys.contains(&id));
                }
            }

            error!("Group channel was closed unexpected");
            flag_pointer.store(false, Ordering::SeqCst);
        }));
    }

    async fn ensure_active(&self) -> Result<(), WsError> {
        match self.active_flag.load(Ordering::SeqCst) {
            true => Ok(()),
            false => Err(WsError::ChannelClosed(ChannelType::Group)),
        }
    }

    pub async fn add_client(&mut self, client: Client) -> Result<(), WsError> {
        self.ensure_active().await?;
        let mut lock = self.clients.lock().await;
        lock.insert(client.id, client);
        Ok(())
    }

    pub async fn add_to_queue(&self, message: Arc<Message>) -> Result<(), WsError> {
        self.ensure_active().await?;
        let writer_clone = self.channel_writer.clone();

        if let Err(e) = writer_clone.send(message).await {
            error!("Group channel is down, error: {}", e);
            return Err(WsError::ChannelError(ChannelType::Group, e));
        };

        Ok(())
    }

    /* Cleanup */

    pub async fn purge_client(&mut self, client_id: Uuid) {
        let mut clients_lock = self.clients.lock().await;

        if let Some(client) = clients_lock.get_mut(&client_id) {
            client.purge();
            clients_lock.retain(|k, _v| *k != client_id);
        }
    }

    pub async fn purge_group_and_clients(&mut self) {
        let mut clients_lock = self.clients.lock().await;

        for (_id, client) in clients_lock.iter_mut() {
            client.purge();
        }

        clients_lock.clear();

        if let Some(task) = self.group_writer_task.take() {
            task.abort();
        };
    }
}
