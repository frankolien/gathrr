use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

#[derive(Debug, Clone)]
pub struct ExportedEvent {
    pub id: Uuid,
    pub title: String,
    pub starts_at: OffsetDateTime,
    pub status: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct ExportedRsvp {
    pub event_id: Uuid,
    pub event_title: String,
    pub status: String,
    pub plus_ones: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct ExportedMessage {
    pub event_id: Uuid,
    pub event_title: String,
    pub seq: i64,
    pub body: String,
    pub created_at: OffsetDateTime,
}

