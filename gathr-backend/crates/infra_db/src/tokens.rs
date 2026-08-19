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

pub async fn burn_refresh(db: &Db, jti: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE refresh_tokens SET used_at = now() WHERE jti = $1 AND used_at IS NULL"#,
        jti
    )
    .execute(db)
    .await
    .map_err(DbError::from_sqlx)?;
    Ok(())
}

pub async fn revoke_family(db: &Db, family_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE refresh_tokens SET revoked_at = now()
           WHERE family_id = $1 AND revoked_at IS NULL"#,
        family_id
    )
    .execute(db)
    .await
    .map_err(DbError::from_sqlx)?;
    Ok(())
}

pub async fn insert_guest_session(
    tx: &mut Tx<'_>,
    user_id: Uuid,
    token_hash: &str,
    invite_id: Option<Uuid>,
    expires_at: OffsetDateTime,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO guest_sessions (user_id, token_hash, invite_id, expires_at)
           VALUES ($1, $2, $3, $4)"#,
        user_id,
        token_hash,
        invite_id,
        expires_at
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;
    Ok(())
}

