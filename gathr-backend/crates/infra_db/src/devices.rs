use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub apns_token: String,
    pub environment: String,
}

pub async fn upsert(
    db: &Db,
    user_id: Uuid,
    apns_token: &str,
    environment: &str,
) -> Result<DeviceRecord, DbError> {
    sqlx::query_as!(
        DeviceRecord,
        r#"INSERT INTO devices (user_id, apns_token, environment)
           VALUES ($1, $2, $3)
           ON CONFLICT (apns_token)
           DO UPDATE SET user_id = $1, environment = $3, last_seen_at = now()
           RETURNING id, user_id, apns_token, environment"#,
        user_id,
        apns_token,
        environment
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn remove(db: &Db, id: Uuid, user_id: Uuid) -> Result<bool, DbError> {
    sqlx::query!(
        r#"DELETE FROM devices WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected() > 0)
    .map_err(DbError::from_sqlx)
}

pub async fn audience_for_event(db: &Db, event_id: Uuid) -> Result<Vec<DeviceRecord>, DbError> {
    sqlx::query_as!(
        DeviceRecord,
        r#"SELECT d.id, d.user_id, d.apns_token, d.environment
           FROM devices d
           WHERE d.user_id IN (
             SELECT r.user_id FROM rsvps r
             WHERE r.event_id = $1 AND r.status IN ('going', 'maybe')
             UNION
             SELECT h.user_id FROM event_hosts h WHERE h.event_id = $1
           )
           AND NOT EXISTS (
             SELECT 1 FROM event_mutes m
             WHERE m.user_id = d.user_id AND m.event_id = $1
           )"#,
        event_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn set_mute(db: &Db, user_id: Uuid, event_id: Uuid, muted: bool) -> Result<(), DbError> {
    if muted {
        sqlx::query!(
            r#"INSERT INTO event_mutes (user_id, event_id) VALUES ($1, $2)
               ON CONFLICT DO NOTHING"#,
            user_id,
            event_id
        )
        .execute(db)
        .await
    } else {
        sqlx::query!(
            r#"DELETE FROM event_mutes WHERE user_id = $1 AND event_id = $2"#,
            user_id,
            event_id
        )
        .execute(db)
        .await
    }
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}
