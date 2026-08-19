use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};
use crate::records::InviteRecord;

pub async fn insert(
    db: &Db,
    event_id: Uuid,
    code: &str,
    max_uses: Option<i32>,
    expires_at: Option<OffsetDateTime>,
    created_by: Uuid,
) -> Result<InviteRecord, DbError> {
    sqlx::query_as!(
        InviteRecord,
        r#"INSERT INTO invites (event_id, code, max_uses, expires_at, created_by)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, event_id, code, max_uses, uses, expires_at"#,
        event_id,
        code,
        max_uses,
        expires_at,
        created_by
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn find_by_code(db: &Db, code: &str) -> Result<Option<InviteRecord>, DbError> {
    sqlx::query_as!(
        InviteRecord,
        r#"SELECT id, event_id, code, max_uses, uses, expires_at
           FROM invites WHERE code = $1"#,
        code
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn lock_by_code(tx: &mut Tx<'_>, code: &str) -> Result<Option<InviteRecord>, DbError> {
    sqlx::query_as!(
        InviteRecord,
        r#"SELECT id, event_id, code, max_uses, uses, expires_at
           FROM invites WHERE code = $1 FOR UPDATE"#,
        code
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}

