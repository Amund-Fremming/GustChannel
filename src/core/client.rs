use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, stream::SplitSink};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::error;
use uuid::Uuid;

use crate::error::{ChannelType, WsError};

static BUFFER_SIZE: usize = 32;

#[derive(Debug)]
pub struct Client {
    pub id: Uuid,
    channel_writer: mpsc::Sender<Arc<Message>>,
    client_writer_task: Option<JoinHandle<()>>,
    active_flag: Arc<AtomicBool>,
}

impl Client {
    pub fn new(id: Uuid, ws_writer: SplitSink<WebSocket, Message>) -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
        let mut client = Self {
            id,
            channel_writer: tx,
            client_writer_task: None,
            active_flag: Arc::new(AtomicBool::new(true)),
        };
        client.spawn_client_writer(rx, ws_writer);
        client
    }

    pub async fn write_to_channel(&mut self, message: Arc<Message>) -> Result<(), WsError> {
        if !self.active_flag.load(Ordering::SeqCst) {
            return Err(WsError::ChannelClosed(ChannelType::Client));
        }

        if let Err(e) = self.channel_writer.send(message).await {
            return Err(WsError::ChannelError(ChannelType::Client, e));
        }

        Ok(())
    }

    fn spawn_client_writer(
        &mut self,
        mut channel_receiver: mpsc::Receiver<Arc<Message>>,
        mut ws_writer: SplitSink<WebSocket, Message>,
    ) {
        let id_clone = self.id.clone();
        let active_pointer = self.active_flag.clone();

        self.client_writer_task = Some(tokio::task::spawn(async move {
            while let Some(message) = channel_receiver.recv().await {
                if let Err(e) = ws_writer.send((*message).clone()).await {
                    error!("Error writing to client: {}, client_id: {}", e, id_clone);
                }
            }

            error!("Client channel was closed unexpected");
            active_pointer.store(false, Ordering::SeqCst);
        }));
    }

    // Cleanup
    pub(crate) fn purge(&mut self) {
        if let Some(task) = self.client_writer_task.take() {
            task.abort();
        }
    }
}
