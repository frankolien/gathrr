use gathr_domain::RsvpStatus;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};
use crate::records::{parse_rsvp_status, GuestRecord, RsvpRecord};

pub async fn seats_held_excluding(
    tx: &mut Tx<'_>,
    event_id: Uuid,
    actor_id: Uuid,
) -> Result<i32, DbError> {
    let row = sqlx::query!(
        r#"SELECT COALESCE(SUM(1 + plus_ones), 0)::int AS "seats!"
           FROM rsvps
           WHERE event_id = $1 AND status = 'going' AND user_id <> $2"#,
        event_id,
        actor_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    Ok(row.seats)
}

pub async fn going_guest_count(db: &Db, event_id: Uuid) -> Result<i32, DbError> {
    let row = sqlx::query!(
        r#"SELECT COUNT(*)::int AS "count!" FROM rsvps
           WHERE event_id = $1 AND status = 'going'"#,
        event_id
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)?;

    Ok(row.count)
}

pub async fn find_in_tx(
    tx: &mut Tx<'_>,
    event_id: Uuid,
    user_id: Uuid,
) -> Result<Option<RsvpRecord>, DbError> {
    let row = sqlx::query!(
        r#"SELECT event_id, user_id, status::text AS "status!", plus_ones, updated_at
           FROM rsvps WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        user_id
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    row.map(|row| {
        Ok(RsvpRecord {
            event_id: row.event_id,
            user_id: row.user_id,
            status: parse_rsvp_status(&row.status)?,
            plus_ones: row.plus_ones,
            updated_at: row.updated_at,
        })
    })
    .transpose()
}

pub async fn find(db: &Db, event_id: Uuid, user_id: Uuid) -> Result<Option<RsvpRecord>, DbError> {
    let row = sqlx::query!(
        r#"SELECT event_id, user_id, status::text AS "status!", plus_ones, updated_at
           FROM rsvps WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)?;

    row.map(|row| {
        Ok(RsvpRecord {
            event_id: row.event_id,
            user_id: row.user_id,
            status: parse_rsvp_status(&row.status)?,
            plus_ones: row.plus_ones,
            updated_at: row.updated_at,
        })
    })
    .transpose()
}

pub async fn upsert(
    tx: &mut Tx<'_>,
    event_id: Uuid,
    user_id: Uuid,
    status: RsvpStatus,
    plus_ones: i32,
    invite_id: Option<Uuid>,
) -> Result<RsvpRecord, DbError> {
    let row = sqlx::query!(
        r#"INSERT INTO rsvps (event_id, user_id, status, plus_ones, invite_id)
           VALUES ($1, $2, ($3::text)::rsvp_status, $4, $5)
           ON CONFLICT (event_id, user_id) DO UPDATE
             SET status = EXCLUDED.status,
                 plus_ones = EXCLUDED.plus_ones,
                 invite_id = COALESCE(rsvps.invite_id, EXCLUDED.invite_id),
                 updated_at = now()
           RETURNING event_id, user_id, status::text AS "status!", plus_ones, updated_at"#,
        event_id,
        user_id,
        status.as_str(),
        plus_ones,
        invite_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    Ok(RsvpRecord {
        event_id: row.event_id,
        user_id: row.user_id,
        status: parse_rsvp_status(&row.status)?,
        plus_ones: row.plus_ones,
        updated_at: row.updated_at,
    })
}

pub async fn list_guests(db: &Db, event_id: Uuid) -> Result<Vec<GuestRecord>, DbError> {
    let rows = sqlx::query!(
        r#"SELECT r.user_id, u.display_name, r.status::text AS "status!", r.plus_ones
           FROM rsvps r
           JOIN users u ON u.id = r.user_id
           WHERE r.event_id = $1
           ORDER BY r.updated_at"#,
        event_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)?;

    rows.into_iter()
        .map(|row| {
            Ok(GuestRecord {
                user_id: row.user_id,
                display_name: row.display_name,
                status: parse_rsvp_status(&row.status)?,
                plus_ones: row.plus_ones,
            })
        })
        .collect()
}

pub async fn remove(db: &Db, event_id: Uuid, user_id: Uuid) -> Result<bool, DbError> {
    sqlx::query!(
        r#"DELETE FROM rsvps WHERE event_id = $1 AND user_id = $2"#,
        event_id,
        user_id
    )
    .execute(db)
    .await
    .map(|done| done.rows_affected() > 0)
    .map_err(DbError::from_sqlx)
}
