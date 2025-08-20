use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{StreamExt, stream::SplitSink};
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct Group {
    writers: Vec<SplitSink<WebSocket, Message>>,
    //   message_queue: ...,
    sender: mpsc::Sender<Message>,
    receiver: Arc<mpsc::Receiver<Message>>,
}

impl Group {
    pub fn new() -> Self {
        let (tx, rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(16);
        Self {
            writers: Vec::new(),
            sender: tx,
            receiver: Arc::new(rx),
        }
    }

    fn spawn_broadcaster(&self) {
        //
        //let m = self.queue.recv().await.unwrap();
    }

    pub fn add(&mut self, socket: WebSocket) {
        let (writer, mut reader) = socket.split();
        self.writers.push(writer);

        let sender_clone = self.sender.clone();

        tokio::task::spawn(async move {
            while let Some(msg) = reader.next().await {
                if let Ok(msg) = msg {
                    // TODO - handle error
                    sender_clone.send(msg).await.expect("Handle this please");
                }
            }
        });
    }
}
