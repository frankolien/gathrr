use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};

#[derive(Debug, Clone)]
pub struct RefreshRecord {
    pub jti: Uuid,
    pub family_id: Uuid,
    pub user_id: Uuid,
    pub used_at: Option<OffsetDateTime>,
    pub revoked_at: Option<OffsetDateTime>,
    pub expires_at: OffsetDateTime,
}

pub async fn insert_refresh(
    db: &Db,
    jti: Uuid,
    family_id: Uuid,
    user_id: Uuid,
    token_hash: &str,
    expires_at: OffsetDateTime,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO refresh_tokens (jti, family_id, user_id, token_hash, expires_at)
           VALUES ($1, $2, $3, $4, $5)"#,
        jti,
        family_id,
        user_id,
        token_hash,
        expires_at
    )
    .execute(db)
    .await
    .map_err(DbError::from_sqlx)?;
    Ok(())
}

pub async fn find_refresh(db: &Db, token_hash: &str) -> Result<Option<RefreshRecord>, DbError> {
    sqlx::query_as!(
        RefreshRecord,
        r#"SELECT jti, family_id, user_id, used_at, revoked_at, expires_at
           FROM refresh_tokens WHERE token_hash = $1"#,
        token_hash
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

