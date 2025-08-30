use std::sync::Arc;

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

use crate::{ChannelError, Client};

static BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Group {
    clients: Arc<Mutex<Vec<Client>>>,
    channel_writer: mpsc::Sender<Message>,
    group_writer_task: Option<JoinHandle<()>>,
}

impl Group {
    pub fn new(client: Client) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);

        let mut group = Self {
            clients: Arc::new(Mutex::new(Vec::from([client]))),
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

    pub fn close(&mut self) {
        if let Some(task) = self.group_writer_task.take() {
            task.abort();
        }
    }

    fn spawn_broadcaster(&mut self, mut receiver: mpsc::Receiver<Message>) {
        let clients_pointer = self.clients.clone();

        self.group_writer_task = Some(tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let mut lock = clients_pointer.lock().await;
                let mut i = 0;

                while i < lock.len() {
                    if let Err(e) = lock[i].add_to_queue(message.clone()).await {
                        error!("{}", e);
                        lock.remove(i);
                        continue;
                    }

                    i += 1;
                }
            }

            // TODO - remove all clients
            // Close group
            // Delete group
        }));
    }

    pub async fn remove(&mut self, client_id: Uuid) {
        let mut lock = self.clients.lock().await;
        lock.retain(|c| c.id != client_id);
    }

    pub async fn add(&mut self, client: Client) {
        let mut lock = self.clients.lock().await;
        lock.push(client);
    }

    pub async fn add_to_queue(&self, message: Message) -> Result<(), ChannelError> {
        let writer_clone = self.channel_writer.clone();

        if let Err(e) = writer_clone.send(message).await {
            error!("Group channel is down, error: {}", e);
            return Err(ChannelError::ChannelError(crate::ChannelType::Group, e));
        };

        Ok(())
    }

    pub async fn close_group_on_channel_failure(&mut self) {
        if let Some(task) = self.group_writer_task.take() {
            task.abort();
        };
    }

    pub async fn send_to_group_except() {
        todo!();
    }
}
