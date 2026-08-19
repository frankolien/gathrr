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

