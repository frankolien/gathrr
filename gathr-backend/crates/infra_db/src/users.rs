use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};
use crate::records::UserRecord;

pub async fn insert(
    tx: &mut Tx<'_>,
    display_name: &str,
    phone: Option<&str>,
    is_guest: bool,
) -> Result<UserRecord, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"INSERT INTO users (display_name, phone, is_guest)
           VALUES ($1, $2, $3)
           RETURNING id, display_name, phone, is_guest, bio"#,
        display_name,
        phone,
        is_guest
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn find(db: &Db, id: Uuid) -> Result<Option<UserRecord>, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"SELECT id, display_name, phone, is_guest, bio FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn find_claimed_by_phone(db: &Db, phone: &str) -> Result<Option<UserRecord>, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"SELECT id, display_name, phone, is_guest, bio
           FROM users WHERE phone = $1 AND is_guest = false"#,
        phone
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

