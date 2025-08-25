use std::sync::Arc;

use axum::extract::ws::{Message, Utf8Bytes};
use futures_util::SinkExt;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::{Client, core::payload};

#[derive(Debug)]
pub struct Group {
    clients: Arc<Mutex<Vec<Client>>>,
    sender: mpsc::Sender<Message>,
    queue: Option<mpsc::Receiver<Message>>,
}

impl Group {
    pub fn new(client: Client) -> Self {
        let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(16);

        let mut group = Self {
            clients: Arc::new(Mutex::new(Vec::from([client]))),
            sender: tx,
            queue: Some(rx),
        };
        group.spawn_broadcaster();
        group
    }

    fn spawn_broadcaster(&mut self) {
        // TODO - handle error
        let mut queue = self.queue.take().expect("Handle this please 1");
        let clients_pointer = self.clients.clone();

        tokio::spawn(async move {
            while let Some(msg) = queue.recv().await {
                let mut lock = clients_pointer.lock().await;

                for client in lock.iter_mut() {
                    client
                        .writer
                        .send(msg.clone())
                        .await
                        .expect("Handle this please 2");
                }
            }
            // TODO - handle error
            // Should never happen if proper error handling in the channel.send is valdating payload properly
        });
    }

    pub async fn remove(&mut self, client_id: Uuid) {
        let mut lock = self.clients.lock().await;
        lock.retain(|c| c.id != client_id);
    }

    pub async fn add(&mut self, client: Client) {
        let mut lock = self.clients.lock().await;
        lock.push(client);
    }

    pub async fn send_to_group(&self, payload: String) {
        let sender_clone = self.sender.clone();

        let message = Message::Text(Utf8Bytes::from(payload));
        if let Err(e) = sender_clone.send(message).await {
            println!("Failed bad: {}", e);
        };
    }

    pub async fn send_to_group_except() {
        todo!();
    }
}
