use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Tx;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Moved {
    pub rsvps: u64,
    pub messages: u64,
}

pub async fn serialize_on(tx: &mut Tx<'_>, shadow: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))"#,
        shadow.to_string()
    )
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn absorb(tx: &mut Tx<'_>, shadow: Uuid, keeper: Uuid) -> Result<Moved, DbError> {
    sqlx::query!(
        r#"UPDATE rsvps keep
           SET status = drop.status, plus_ones = drop.plus_ones, updated_at = drop.updated_at
           FROM rsvps drop
           WHERE keep.user_id = $2 AND drop.user_id = $1
             AND keep.event_id = drop.event_id
             AND drop.updated_at > keep.updated_at"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"DELETE FROM rsvps drop
           WHERE drop.user_id = $1
             AND EXISTS (
               SELECT 1 FROM rsvps keep
               WHERE keep.user_id = $2 AND keep.event_id = drop.event_id
             )"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    let rsvps = sqlx::query!(
        r#"UPDATE rsvps SET user_id = $2 WHERE user_id = $1"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?
    .rows_affected();

    let messages = sqlx::query!(
        r#"UPDATE messages SET sender_id = $2 WHERE sender_id = $1"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?
    .rows_affected();

    sqlx::query!(
        r#"UPDATE media SET owner_id = $2 WHERE owner_id = $1"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"DELETE FROM event_mutes drop
           WHERE drop.user_id = $1
             AND EXISTS (
               SELECT 1 FROM event_mutes keep
               WHERE keep.user_id = $2 AND keep.event_id = drop.event_id
             )"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"UPDATE event_mutes SET user_id = $2 WHERE user_id = $1"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"DELETE FROM notifications drop
           WHERE drop.user_id = $1
             AND drop.kind = 'message_posted'
             AND drop.read_at IS NULL
             AND EXISTS (
               SELECT 1 FROM notifications keep
               WHERE keep.user_id = $2 AND keep.event_id = drop.event_id
                 AND keep.kind = 'message_posted' AND keep.read_at IS NULL
             )"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"UPDATE notifications SET user_id = $2 WHERE user_id = $1"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"UPDATE notifications SET actor_id = $2 WHERE actor_id = $1"#,
        shadow,
        keeper
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, shadow)
        .execute(&mut **tx)
        .await
        .map_err(DbError::from_sqlx)?;

    Ok(Moved { rsvps, messages })
}
