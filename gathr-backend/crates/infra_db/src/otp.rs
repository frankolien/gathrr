use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

#[derive(Debug, Clone)]
pub struct OtpChallengeRecord {
    pub id: Uuid,
    pub code_hash: String,
    pub attempts: i32,
    pub expires_at: OffsetDateTime,
}

pub async fn supersede_pending(db: &Db, channel: &str, destination: &str) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE otp_challenges SET consumed_at = now()
           WHERE channel = ($1::text)::otp_channel
             AND destination = $2
             AND consumed_at IS NULL"#,
        channel,
        destination
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn insert(
    db: &Db,
    channel: &str,
    destination: &str,
    code_hash: &str,
    expires_at: OffsetDateTime,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO otp_challenges (channel, destination, code_hash, expires_at)
           VALUES (($1::text)::otp_channel, $2, $3, $4)"#,
        channel,
        destination,
        code_hash,
        expires_at
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

