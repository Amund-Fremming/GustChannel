use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::sync::{Mutex, mpsc};

#[derive(Debug)]
pub struct Group {
    clients: Arc<Mutex<Vec<SplitSink<WebSocket, Message>>>>,
    sender: mpsc::Sender<Message>,
    queue: Option<mpsc::Receiver<Message>>,
}

impl Group {
    pub fn new(socket: WebSocket) -> Self {
        let (writer, reader) = socket.split();

        let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(16);
        let sender_clone = tx.clone();

        let mut group = Self {
            clients: Arc::new(Mutex::new(Vec::from([writer]))),
            sender: tx,
            queue: Some(rx),
        };
        group.spawn_broadcaster();

        Self::spawn_listener(sender_clone, reader);
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
                        .send(msg.clone())
                        .await
                        .expect("Handle this please 2");
                }
            }
            // TODO - handle error
            // Should never happen if proper error handling in the channel.send is valdating payload properly
        });
    }

    pub async fn add(&mut self, socket: WebSocket) {
        let (writer, reader) = socket.split();

        let mut lock = self.clients.lock().await;
        lock.push(writer);

        let channel_clone = self.sender.clone();
        Self::spawn_listener(channel_clone, reader);
    }

    pub async fn send_to_group(&self, message: Message) {
        if let Err(e) = self.sender.send(message).await {
            println!("Failed bad: {}", e);
        };
    }

    pub async fn send_to_group_except() {
        todo!();
    }

    // remove actuallt
    fn spawn_listener(sender_clone: mpsc::Sender<Message>, mut reader: SplitStream<WebSocket>) {
        tokio::task::spawn(async move {
            while let Some(msg) = reader.next().await {
                if let Ok(msg) = msg {
                    // TODO - handle error

                    println!("Group received message: {:?}", msg);
                    if let Err(e) = sender_clone.send(msg).await {
                        println!("Failed bad: {}", e);
                    }
                }
            }
            // TODO - handle error
        });
    }
}
