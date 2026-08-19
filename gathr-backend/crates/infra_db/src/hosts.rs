use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};

pub const OWNER: &str = "owner";
pub const CO_HOST: &str = "co_host";

#[derive(Debug, Clone)]
pub struct HostRecord {
    pub user_id: Uuid,
    pub display_name: String,
    pub role: String,
    pub added_at: OffsetDateTime,
}

pub async fn install_owner(tx: &mut Tx<'_>, event_id: Uuid, user_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"INSERT INTO event_hosts (event_id, user_id, role, added_by)
           VALUES ($1, $2, 'owner', $2)
           ON CONFLICT DO NOTHING"#,
        event_id,
        user_id
    )
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn manages(db: &Db, event_id: Uuid, user_id: Uuid) -> Result<bool, DbError> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM event_hosts WHERE event_id = $1 AND user_id = $2
           ) AS "manages!""#,
        event_id,
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn role_of(db: &Db, event_id: Uuid, user_id: Uuid) -> Result<Option<String>, DbError> {
    sqlx::query_scalar!(
        r#"SELECT role::text AS "role!" FROM event_hosts
           WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn list(db: &Db, event_id: Uuid) -> Result<Vec<HostRecord>, DbError> {
    sqlx::query_as!(
        HostRecord,
        r#"SELECT h.user_id, u.display_name, h.role::text AS "role!", h.added_at
           FROM event_hosts h
           JOIN users u ON u.id = h.user_id
           WHERE h.event_id = $1
           ORDER BY h.role, h.added_at"#,
        event_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn add(db: &Db, event_id: Uuid, user_id: Uuid, added_by: Uuid) -> Result<bool, DbError> {
    sqlx::query!(
        r#"INSERT INTO event_hosts (event_id, user_id, role, added_by)
           VALUES ($1, $2, 'co_host', $3)
           ON CONFLICT DO NOTHING"#,
        event_id,
        user_id,
        added_by
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected() > 0)
    .map_err(DbError::from_sqlx)
}

pub async fn lock_roster(tx: &mut Tx<'_>, event_id: Uuid) -> Result<Vec<(Uuid, String)>, DbError> {
    let rows = sqlx::query!(
        r#"SELECT user_id, role::text AS "role!" FROM event_hosts
           WHERE event_id = $1
           ORDER BY user_id
           FOR UPDATE"#,
        event_id
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    Ok(rows
        .into_iter()
        .map(|row| (row.user_id, row.role))
        .collect())
}

pub async fn remove(tx: &mut Tx<'_>, event_id: Uuid, user_id: Uuid) -> Result<bool, DbError> {
    sqlx::query!(
        r#"DELETE FROM event_hosts WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        user_id
    )
    .execute(&mut **tx)
    .await
    .map(|done| done.rows_affected() > 0)
    .map_err(DbError::from_sqlx)
}

pub async fn hand_over(tx: &mut Tx<'_>, event_id: Uuid, new_owner: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"DELETE FROM event_hosts
           WHERE event_id = $1 AND role = 'owner'"#,
        event_id
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"UPDATE event_hosts SET role = 'owner', added_at = now()
           WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        new_owner
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"UPDATE events SET host_id = $2, updated_at = now() WHERE id = $1"#,
        event_id,
        new_owner
    )
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(DbError::from_sqlx)
}

pub async fn events_needing_a_new_owner(db: &Db, user_id: Uuid) -> Result<Vec<Uuid>, DbError> {
    sqlx::query_scalar!(
        r#"SELECT event_id FROM event_hosts
           WHERE user_id = $1 AND role = 'owner'"#,
        user_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn successor(
    tx: &mut Tx<'_>,
    event_id: Uuid,
    leaving: Uuid,
) -> Result<Option<Uuid>, DbError> {
    sqlx::query_scalar!(
        r#"SELECT user_id FROM event_hosts
           WHERE event_id = $1 AND user_id <> $2
           ORDER BY added_at
           LIMIT 1"#,
        event_id,
        leaving
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)
}
