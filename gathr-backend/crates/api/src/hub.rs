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

