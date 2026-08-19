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

