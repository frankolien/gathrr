use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

#[derive(Debug, Clone)]
pub struct NotificationRecord {
    pub id: Uuid,
    pub kind: String,
    pub event_id: Uuid,
    pub event_title: String,
    pub actor_display_name: Option<String>,
    pub read_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

pub async fn notify_host(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
    kind: &str,
) -> Result<u64, DbError> {
    sqlx::query!(
        r#"INSERT INTO notifications (user_id, actor_id, event_id, kind)
           SELECT e.host_id, $2, e.id, ($3::text)::notification_kind
           FROM events e
           WHERE e.id = $1
             AND e.host_id <> $2
             AND NOT EXISTS (
               SELECT 1 FROM event_mutes m
               WHERE m.user_id = e.host_id AND m.event_id = e.id
             )"#,
        event_id,
        actor_id,
        kind
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected())
    .map_err(DbError::from_sqlx)
}

pub async fn notify_members(
    db: &Db,
    event_id: Uuid,
    actor_id: Option<Uuid>,
    kind: &str,
) -> Result<u64, DbError> {
    sqlx::query!(
        r#"INSERT INTO notifications (user_id, actor_id, event_id, kind)
           SELECT audience.user_id, $2, $1, ($3::text)::notification_kind
           FROM (
             SELECT r.user_id FROM rsvps r
             WHERE r.event_id = $1 AND r.status <> 'declined'
             UNION
             SELECT e.host_id FROM events e WHERE e.id = $1
           ) AS audience
           WHERE audience.user_id IS DISTINCT FROM $2
             AND NOT EXISTS (
               SELECT 1 FROM event_mutes m
               WHERE m.user_id = audience.user_id AND m.event_id = $1
             )"#,
        event_id,
        actor_id,
        kind
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected())
    .map_err(DbError::from_sqlx)
}

