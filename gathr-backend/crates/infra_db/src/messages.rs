use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};

#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub sender_id: Uuid,
    pub sender_display_name: String,
    pub seq: i64,
    pub body: String,
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
                  u.display_name AS sender_display_name,
                  inserted.seq, inserted.body, inserted.created_at
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

