use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};

#[derive(Debug, Clone)]
pub struct ReminderRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub kind: String,
    pub attempts: i32,
}

pub async fn schedule(
    db: &Db,
    event_id: Uuid,
    kind: &str,
    run_at: OffsetDateTime,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO reminder_jobs (event_id, kind, run_at)
           VALUES ($1, $2, $3)
           ON CONFLICT (event_id, kind)
           DO UPDATE SET run_at = $3, status = 'pending', attempts = 0, last_error = NULL
           WHERE reminder_jobs.status <> 'sent'"#,
        event_id,
        kind,
        run_at
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn cancel_for_event(db: &Db, event_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE reminder_jobs SET status = 'cancelled'
           WHERE event_id = $1 AND status = 'pending'"#,
        event_id
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn claim_due(
    tx: &mut Tx<'_>,
    now: OffsetDateTime,
    limit: i64,
) -> Result<Vec<ReminderRecord>, DbError> {
    sqlx::query_as!(
        ReminderRecord,
        r#"UPDATE reminder_jobs SET status = 'running', locked_at = now()
           WHERE id IN (
             SELECT id FROM reminder_jobs
             WHERE status = 'pending' AND run_at <= $1
             ORDER BY run_at
             FOR UPDATE SKIP LOCKED
             LIMIT $2
           )
           RETURNING id, event_id, kind, attempts"#,
        now,
        limit
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn mark_sent(db: &Db, id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE reminder_jobs SET status = 'sent', locked_at = NULL WHERE id = $1"#,
        id
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn mark_failed(
    db: &Db,
    id: Uuid,
    reason: &str,
    max_attempts: i32,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE reminder_jobs
           SET attempts = attempts + 1,
               locked_at = NULL,
               last_error = $2,
               status = CASE WHEN attempts + 1 >= $3 THEN 'failed' ELSE 'pending' END
           WHERE id = $1"#,
        id,
        reason,
        max_attempts
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

