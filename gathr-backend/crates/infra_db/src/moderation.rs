use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

pub async fn file_report(
    db: &Db,
    reporter_id: Uuid,
    subject: &str,
    subject_id: Uuid,
    event_id: Option<Uuid>,
    reason: &str,
    detail: Option<&str>,
) -> Result<Uuid, DbError> {
    sqlx::query_scalar!(
        r#"INSERT INTO reports (reporter_id, subject, subject_id, event_id, reason, detail)
           VALUES ($1, ($2::text)::report_subject, $3, $4, $5, $6)
           ON CONFLICT (reporter_id, subject, subject_id)
           DO UPDATE SET reason = $5, detail = $6, created_at = now()
           RETURNING id"#,
        reporter_id,
        subject,
        subject_id,
        event_id,
        reason,
        detail
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn block(db: &Db, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO blocks (blocker_id, blocked_id) VALUES ($1, $2)
           ON CONFLICT DO NOTHING"#,
        blocker_id,
        blocked_id
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn unblock(db: &Db, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, DbError> {
    sqlx::query!(
        r#"DELETE FROM blocks WHERE blocker_id = $1 AND blocked_id = $2"#,
        blocker_id,
        blocked_id
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected() > 0)
    .map_err(DbError::from_sqlx)
}

