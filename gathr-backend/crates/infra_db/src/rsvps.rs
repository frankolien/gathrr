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

