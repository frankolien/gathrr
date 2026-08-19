use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};
use crate::records::UserRecord;

pub async fn find_user(
    db: &Db,
    provider: &str,
    subject: &str,
) -> Result<Option<UserRecord>, DbError> {
    sqlx::query_as!(
        UserRecord,
        r#"SELECT u.id, u.display_name, u.phone, u.is_guest, u.bio
           FROM identities i
           JOIN users u ON u.id = i.user_id
           WHERE i.provider = ($1::text)::identity_provider AND i.subject = $2"#,
        provider,
        subject
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn link(
    tx: &mut Tx<'_>,
    user_id: Uuid,
    provider: &str,
    subject: &str,
    email: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO identities (user_id, provider, subject, email)
           VALUES ($1, ($2::text)::identity_provider, $3, $4)
           ON CONFLICT (provider, subject) DO NOTHING"#,
        user_id,
        provider,
        subject,
        email
    )
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}
