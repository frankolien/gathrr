use std::collections::HashMap;
use std::sync::Arc;

use actix_ws::Session;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct EventHub {
    rooms: Arc<Mutex<HashMap<Uuid, Vec<Subscriber>>>>,
}

struct Subscriber {
    id: Uuid,
    session: Session,
}

impl EventHub {
    pub async fn join(&self, event_id: Uuid, session: Session) -> Uuid {
        let id = Uuid::new_v4();
        self.rooms
            .lock()
            .await
            .entry(event_id)
            .or_default()
            .push(Subscriber { id, session });
        id
    }

    pub async fn leave(&self, event_id: Uuid, subscriber_id: Uuid) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&event_id) {
            room.retain(|subscriber| subscriber.id != subscriber_id);
            if room.is_empty() {
                rooms.remove(&event_id);
            }
        }
    }

    pub async fn broadcast(&self, event_id: Uuid, payload: &str) {
        let mut rooms = self.rooms.lock().await;
        let Some(room) = rooms.get_mut(&event_id) else {
            return;
        };

        let mut delivered = Vec::with_capacity(room.len());
        for mut subscriber in room.drain(..) {
            if subscriber.session.text(payload).await.is_ok() {
                delivered.push(subscriber);
            }
        }

        if delivered.is_empty() {
            rooms.remove(&event_id);
        } else {
            *room = delivered;
        }
    }

    pub async fn subscriber_count(&self, event_id: Uuid) -> usize {
        self.rooms
            .lock()
            .await
            .get(&event_id)
            .map_or(0, |room| room.len())
    }
}
