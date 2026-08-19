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

#[derive(Debug, Clone)]
pub struct ExportedIdentity {
    pub provider: String,
    pub created_at: OffsetDateTime,
}

pub async fn hosted_events(db: &Db, user_id: Uuid) -> Result<Vec<ExportedEvent>, DbError> {
    sqlx::query_as!(
        ExportedEvent,
        r#"SELECT id, title, starts_at, status::text AS "status!", created_at
           FROM events WHERE host_id = $1 ORDER BY created_at"#,
        user_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn rsvps(db: &Db, user_id: Uuid) -> Result<Vec<ExportedRsvp>, DbError> {
    sqlx::query_as!(
        ExportedRsvp,
        r#"SELECT r.event_id, e.title AS event_title, r.status::text AS "status!",
                  r.plus_ones, r.created_at
           FROM rsvps r JOIN events e ON e.id = r.event_id
           WHERE r.user_id = $1 ORDER BY r.created_at"#,
        user_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

