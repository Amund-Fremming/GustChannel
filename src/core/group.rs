use std::sync::Arc;

use axum::extract::ws::Message;
use futures_util::SinkExt;
use tokio::{
    sync::{
        Mutex,
        mpsc::{self},
    },
    task::JoinHandle,
};
use uuid::Uuid;

use crate::Client;

static BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Group {
    clients: Arc<Mutex<Vec<Client>>>,
    sender: mpsc::Sender<Message>,
    broadcast_task: Option<JoinHandle<()>>,
}

impl Group {
    pub fn new(client: Client) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);

        let mut group = Self {
            clients: Arc::new(Mutex::new(Vec::from([client]))),
            sender: tx,
            broadcast_task: None,
        };
        group.spawn_broadcaster(rx);
        group
    }

    pub async fn empty(&self) -> bool {
        let lock = self.clients.lock().await;
        lock.len() == 0
    }

    pub fn close(&mut self) {
        if let Some(task) = self.broadcast_task.take() {
            task.abort();
        }
    }

    fn spawn_broadcaster(&mut self, mut queue: mpsc::Receiver<Message>) {
        let clients_pointer = self.clients.clone();

        self.broadcast_task = Some(tokio::spawn(async move {
            while let Some(message_type) = queue.recv().await {
                let mut lock = clients_pointer.lock().await;
                let mut i = 0;

                while i < lock.len() {
                    if let Err(_) = lock[i].writer.send(message_type.clone()).await {
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

    pub async fn send_to_group(&self, message: Message) {
        let sender_clone = self.sender.clone();

        if let Err(e) = sender_clone.send(message).await {
            println!("Failed bad: {}", e);
        };
    }

    pub async fn send_to_group_except() {
        todo!();
    }
}
