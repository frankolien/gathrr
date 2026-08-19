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

