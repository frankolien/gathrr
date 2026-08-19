use gathr_domain::{InviteCode, InviteTerms, CODE_LENGTH};
use gathr_infra_db::{events, invites, users, Db, EventRecord, InviteRecord};
use rand::RngCore;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;
use crate::events::observed_status;

const CODE_ATTEMPTS: u8 = 5;

#[derive(Debug, Clone)]
pub struct PublicInvite {
    pub invite: InviteRecord,
    pub event: EventRecord,
    pub host_first_name: String,
    pub going_guests: i32,
}

pub async fn create(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
    max_uses: Option<i32>,
    expires_at: Option<OffsetDateTime>,
) -> Result<InviteRecord, AppError> {
    let event = events::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if event.host_id != actor_id {
        return Err(AppError::Forbidden);
    }
    if max_uses.is_some_and(|uses| uses <= 0) {
        return Err(AppError::Validation(
            "max_uses must be a positive number".to_owned(),
        ));
    }

    for _ in 0..CODE_ATTEMPTS {
        let code = InviteCode::from_entropy(entropy());
        match invites::insert(db, event_id, code.as_str(), max_uses, expires_at, actor_id).await {
            Ok(record) => return Ok(record),
            Err(error) if error.is_conflict() => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(AppError::CodeExhaustion)
}

pub async fn resolve(db: &Db, raw_code: &str) -> Result<PublicInvite, AppError> {
    let code = InviteCode::parse(raw_code).map_err(|_| AppError::InviteInvalid)?;
    let invite = invites::find_by_code(db, code.as_str())
        .await?
        .ok_or(AppError::InviteInvalid)?;

    terms(&invite).guard_redeemable(OffsetDateTime::now_utc())?;

    let event = events::find(db, invite.event_id)
        .await?
        .ok_or(AppError::InviteInvalid)?;

    let status = observed_status(&event, OffsetDateTime::now_utc());
    gathr_domain::event::guard_accepts_rsvps(status)?;

    let host = users::find(db, event.host_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let going_guests = gathr_infra_db::rsvps::going_guest_count(db, event.id).await?;

    Ok(PublicInvite {
        host_first_name: first_name(&host.display_name),
        invite,
        event,
        going_guests,
    })
}

pub async fn list(db: &Db, event_id: Uuid, actor_id: Uuid) -> Result<Vec<InviteRecord>, AppError> {
    let event = events::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if event.host_id != actor_id {
        return Err(AppError::Forbidden);
    }
    Ok(invites::list_for_event(db, event_id).await?)
}

pub fn terms(invite: &InviteRecord) -> InviteTerms {
    InviteTerms {
        max_uses: invite.max_uses,
        uses: invite.uses,
        expires_at: invite.expires_at,
    }
}

