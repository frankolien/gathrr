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

pub struct ProfileEdit<'a> {
    pub display_name: Option<&'a str>,
    pub bio: Option<Option<&'a str>>,
    pub avatar_media_id: Option<Uuid>,
}

pub async fn update_profile(
    db: &Db,
    user_id: Uuid,
    edit: ProfileEdit<'_>,
) -> Result<Option<UserRecord>, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"UPDATE users SET
             display_name    = COALESCE($2, display_name),
             bio             = CASE WHEN $3 THEN $4 ELSE bio END,
             avatar_media_id = COALESCE($5, avatar_media_id)
           WHERE id = $1
           RETURNING id, display_name, phone, is_guest, bio"#,
        user_id,
        edit.display_name,
        edit.bio.is_some(),
        edit.bio.flatten(),
        edit.avatar_media_id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn find_claimed_by_email(db: &Db, email: &str) -> Result<Option<UserRecord>, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"SELECT id, display_name, phone, is_guest, bio
           FROM users WHERE email = $1 AND is_guest = false"#,
        email
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn insert_verified(
    tx: &mut Tx<'_>,
    display_name: &str,
    email: Option<&str>,
    phone: Option<&str>,
) -> Result<UserRecord, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"INSERT INTO users (display_name, email, phone, is_guest)
           VALUES ($1, $2, $3, false)
           RETURNING id, display_name, phone, is_guest, bio"#,
        display_name,
        email,
        phone
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn find_in_tx(tx: &mut Tx<'_>, id: Uuid) -> Result<Option<UserRecord>, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"SELECT id, display_name, phone, is_guest, bio FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}
