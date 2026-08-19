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

pub async fn notify_hosts(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
    kind: &str,
) -> Result<u64, DbError> {
    sqlx::query!(
        r#"INSERT INTO notifications (user_id, actor_id, event_id, kind)
           SELECT h.user_id, $2, h.event_id, ($3::text)::notification_kind
           FROM event_hosts h
           WHERE h.event_id = $1
             AND h.user_id <> $2
             AND NOT EXISTS (
               SELECT 1 FROM event_mutes m
               WHERE m.user_id = h.user_id AND m.event_id = h.event_id
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
             SELECT h.user_id FROM event_hosts h WHERE h.event_id = $1
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

pub async fn announce_message(db: &Db, event_id: Uuid, sender_id: Uuid) -> Result<u64, DbError> {
    sqlx::query!(
        r#"INSERT INTO notifications (user_id, actor_id, event_id, kind)
           SELECT audience.user_id, $2, $1, 'message_posted'
           FROM (
             SELECT r.user_id FROM rsvps r
             WHERE r.event_id = $1 AND r.status <> 'declined'
             UNION
             SELECT h.user_id FROM event_hosts h WHERE h.event_id = $1
           ) AS audience
           WHERE audience.user_id <> $2
             AND NOT EXISTS (
               SELECT 1 FROM event_mutes m
               WHERE m.user_id = audience.user_id AND m.event_id = $1
             )
           ON CONFLICT (user_id, event_id)
           WHERE read_at IS NULL AND kind = 'message_posted'
           DO UPDATE SET actor_id = EXCLUDED.actor_id, created_at = now()"#,
        event_id,
        sender_id
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected())
    .map_err(DbError::from_sqlx)
}

pub async fn page(
    db: &Db,
    user_id: Uuid,
    before: Option<OffsetDateTime>,
    limit: i64,
) -> Result<Vec<NotificationRecord>, DbError> {
    sqlx::query_as!(
        NotificationRecord,
        r#"SELECT n.id,
                  n.kind::text AS "kind!",
                  n.event_id,
                  e.title AS event_title,
                  a.display_name AS "actor_display_name?",
                  n.read_at,
                  n.created_at
           FROM notifications n
           JOIN events e ON e.id = n.event_id
           LEFT JOIN users a ON a.id = n.actor_id
           WHERE n.user_id = $1
             AND ($2::timestamptz IS NULL OR n.created_at < $2)
           ORDER BY n.created_at DESC
           LIMIT $3"#,
        user_id,
        before,
        limit
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn unread_count(db: &Db, user_id: Uuid) -> Result<i64, DbError> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM notifications
           WHERE user_id = $1 AND read_at IS NULL"#,
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn mark_read(db: &Db, user_id: Uuid, ids: &[Uuid]) -> Result<u64, DbError> {
    sqlx::query!(
        r#"UPDATE notifications SET read_at = now()
           WHERE user_id = $1
             AND read_at IS NULL
             AND (cardinality($2::uuid[]) = 0 OR id = ANY($2))"#,
        user_id,
        ids
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected())
    .map_err(DbError::from_sqlx)
}
