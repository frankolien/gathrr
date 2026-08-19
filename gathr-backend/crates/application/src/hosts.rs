use gathr_infra_db::hosts::{self, HostRecord, CO_HOST, OWNER};
use gathr_infra_db::{events, users, Db, DbError};
use uuid::Uuid;

use crate::error::AppError;

pub async fn roster(db: &Db, event_id: Uuid, actor_id: Uuid) -> Result<Vec<HostRecord>, AppError> {
    crate::messages::authorize_read(db, event_id, actor_id).await?;
    Ok(hosts::list(db, event_id).await?)
}

pub async fn invite(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
    invitee_id: Uuid,
) -> Result<Vec<HostRecord>, AppError> {
    events::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if !hosts::manages(db, event_id, actor_id).await? {
        return Err(AppError::Forbidden);
    }
    users::find(db, invitee_id)
        .await?
        .ok_or(AppError::NotFound)?;

    hosts::add(db, event_id, invitee_id, actor_id).await?;
    Ok(hosts::list(db, event_id).await?)
}

pub async fn stand_down(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<(), AppError> {
    let actor_role = hosts::role_of(db, event_id, actor_id)
        .await?
        .ok_or(AppError::Forbidden)?;
    if actor_role != OWNER && actor_id != target_id {
        return Err(AppError::Forbidden);
    }

    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;
    let roster = hosts::lock_roster(&mut tx, event_id).await?;

    let Some((_, target_role)) = roster.iter().find(|(id, _)| *id == target_id) else {
        return Err(AppError::NotFound);
    };
    if target_role == OWNER {
        return Err(AppError::Validation(
            "the owner cannot step down while they still own the event".to_owned(),
        ));
    }
    if roster.len() <= 1 {
        return Err(AppError::Validation(
            "an event cannot be left without a host".to_owned(),
        ));
    }

    hosts::remove(&mut tx, event_id, target_id).await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;
    Ok(())
}

pub async fn hand_over(db: &Db, event_id: Uuid, leaving: Uuid) -> Result<Option<Uuid>, AppError> {
    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;
    hosts::lock_roster(&mut tx, event_id).await?;

    let Some(successor) = hosts::successor(&mut tx, event_id, leaving).await? else {
        tx.commit().await.map_err(DbError::from_sqlx)?;
        return Ok(None);
    };

    hosts::hand_over(&mut tx, event_id, successor).await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;
    Ok(Some(successor))
}

pub fn is_co_host(role: &str) -> bool {
    role == CO_HOST
}
