use gathr_infra_db::{merge, tokens, users, Db, DbError};
use uuid::Uuid;

use crate::auth::hash_token;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claimed {
    pub rsvps_moved: u64,
    pub messages_moved: u64,
}

pub async fn claim(db: &Db, keeper_id: Uuid, guest_token: &str) -> Result<Claimed, AppError> {
    let keeper = users::find(db, keeper_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if keeper.is_guest {
        return Err(AppError::Forbidden);
    }

    let hash = hash_token(guest_token.trim());
    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;

    let shadow = tokens::find_guest_session_in_tx(&mut tx, &hash)
        .await?
        .ok_or(AppError::GuestSessionInvalid)?;

    merge::serialize_on(&mut tx, shadow).await?;

    if tokens::find_guest_session_in_tx(&mut tx, &hash)
        .await?
        .is_none()
    {
        return Err(AppError::GuestSessionInvalid);
    }

    if shadow == keeper_id {
        tx.commit().await.map_err(DbError::from_sqlx)?;
        return Ok(Claimed {
            rsvps_moved: 0,
            messages_moved: 0,
        });
    }

    let record = users::find_in_tx(&mut tx, shadow)
        .await?
        .ok_or(AppError::GuestSessionInvalid)?;
    if !record.is_guest {
        return Err(AppError::Forbidden);
    }

    let moved = merge::absorb(&mut tx, shadow, keeper_id).await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;

    Ok(Claimed {
        rsvps_moved: moved.rsvps,
        messages_moved: moved.messages,
    })
}
