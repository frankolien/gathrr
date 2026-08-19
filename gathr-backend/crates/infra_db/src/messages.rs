use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};

#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub sender_display_name: Option<String>,
    pub seq: i64,
    pub body: String,
    pub redacted: bool,
    pub created_at: OffsetDateTime,
}

pub async fn allocate_seq(tx: &mut Tx<'_>, event_id: Uuid) -> Result<i64, DbError> {
    sqlx::query_scalar!(
        r#"INSERT INTO event_counters (event_id, last_seq)
           VALUES ($1, 1)
           ON CONFLICT (event_id)
           DO UPDATE SET last_seq = event_counters.last_seq + 1
           RETURNING last_seq"#,
        event_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn insert(
    tx: &mut Tx<'_>,
    event_id: Uuid,
    sender_id: Uuid,
    seq: i64,
    body: &str,
) -> Result<MessageRecord, DbError> {
    sqlx::query_as!(
        MessageRecord,
        r#"WITH inserted AS (
             INSERT INTO messages (event_id, sender_id, seq, body)
             VALUES ($1, $2, $3, $4)
             RETURNING id, event_id, sender_id, seq, body, created_at
           )
           SELECT inserted.id, inserted.event_id, inserted.sender_id,
                  u.display_name AS "sender_display_name?",
                  inserted.seq, inserted.body,
                  false AS "redacted!",
                  inserted.created_at
           FROM inserted JOIN users u ON u.id = inserted.sender_id"#,
        event_id,
        sender_id,
        seq,
        body
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn page(
    db: &Db,
    event_id: Uuid,
    reader_id: Uuid,
    after_seq: i64,
    limit: i64,
) -> Result<Vec<MessageRecord>, DbError> {
    sqlx::query_as!(
        MessageRecord,
        r#"SELECT m.id, m.event_id, m.sender_id,
                  u.display_name AS "sender_display_name?",
                  m.seq,
                  CASE WHEN m.redacted_at IS NULL THEN m.body ELSE '' END AS "body!",
                  (m.redacted_at IS NOT NULL) AS "redacted!",
                  m.created_at
           FROM messages m
           LEFT JOIN users u ON u.id = m.sender_id
           WHERE m.event_id = $1 AND m.seq > $2
             AND NOT EXISTS (
               SELECT 1 FROM blocks b
               WHERE (b.blocker_id = $4 AND b.blocked_id = m.sender_id)
                  OR (b.blocker_id = m.sender_id AND b.blocked_id = $4)
             )
           ORDER BY m.seq ASC
           LIMIT $3"#,
        event_id,
        after_seq,
        limit,
        reader_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn latest_seq(db: &Db, event_id: Uuid) -> Result<i64, DbError> {
    sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(seq), 0) AS "seq!" FROM messages WHERE event_id = $1"#,
        event_id
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn redact_by_sender(db: &Db, sender_id: Uuid) -> Result<u64, DbError> {
    sqlx::query!(
        r#"UPDATE messages SET redacted_at = now(), body = ''
           WHERE sender_id = $1 AND redacted_at IS NULL"#,
        sender_id
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected())
    .map_err(DbError::from_sqlx)
}

pub async fn find(db: &Db, id: Uuid) -> Result<Option<MessageRecord>, DbError> {
    sqlx::query_as!(
        MessageRecord,
        r#"SELECT m.id, m.event_id, m.sender_id,
                  u.display_name AS "sender_display_name?",
                  m.seq,
                  CASE WHEN m.redacted_at IS NULL THEN m.body ELSE '' END AS "body!",
                  (m.redacted_at IS NOT NULL) AS "redacted!",
                  m.created_at
           FROM messages m
           LEFT JOIN users u ON u.id = m.sender_id
           WHERE m.id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}
