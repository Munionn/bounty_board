use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

const ROOM_CAPACITY: usize = 256;

/// In-memory fan-out for chat rooms. One broadcast channel per `chat_id`.
#[derive(Clone, Default)]
pub struct ChatHub {
    rooms: Arc<Mutex<HashMap<Uuid, broadcast::Sender<String>>>>,
}

impl ChatHub {
    pub async fn subscribe(&self, chat_id: Uuid) -> broadcast::Receiver<String> {
        let mut rooms = self.rooms.lock().await;
        let sender = rooms
            .entry(chat_id)
            .or_insert_with(|| broadcast::channel(ROOM_CAPACITY).0);
        sender.subscribe()
    }

    pub async fn publish(&self, chat_id: Uuid, payload: String) {
        let rooms = self.rooms.lock().await;
        if let Some(sender) = rooms.get(&chat_id) {
            let _ = sender.send(payload);
        }
    }
}
