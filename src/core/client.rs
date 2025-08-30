use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, stream::SplitSink};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::error;
use uuid::Uuid;

use crate::ChannelError;

static BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Client {
    pub id: Uuid,
    channel_writer: mpsc::Sender<Message>,
    client_writer_task: Option<JoinHandle<()>>,
}

impl Client {
    pub fn new(id: Uuid, writer: SplitSink<WebSocket, Message>) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
        let mut client = Self {
            id,
            channel_writer: tx,
            client_writer_task: None,
        };
        client.spawn_client_writer(rx, writer);
        client
    }

    pub async fn add_to_queue(&mut self, message: Message) -> Result<(), ChannelError> {
        if let Err(e) = self.channel_writer.send(message).await {
            if let Some(task) = self.client_writer_task.take() {
                task.abort();
            }

            return Err(ChannelError::ChannelError(super::ChannelType::Client, e));
        }

        Ok(())
    }

    fn spawn_client_writer(
        &mut self,
        mut receiver: mpsc::Receiver<Message>,
        mut writer: SplitSink<WebSocket, Message>,
    ) {
        let id_clone = self.id.clone();

        self.client_writer_task = Some(tokio::task::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if let Err(e) = writer.send(message).await {
                    error!("Error writing to client: {}, client_id: {}", e, id_clone);
                }
            }

            error!("Client channel was closed unexpected");
        }));
    }
}
