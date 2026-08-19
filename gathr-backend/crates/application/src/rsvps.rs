use gathr_domain::{event, rsvp, CapacityContext, RsvpRequest, RsvpStatus};
use gathr_infra_db::{events, invites, rsvps, tokens, users, Db, DbError, GuestRecord, Tx};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::auth::{hash_token, random_token};
use crate::error::AppError;
use crate::events::observed_status;
use crate::invites::terms;

#[derive(Debug, Clone)]
pub struct SubmitRsvp {
    pub event_id: Uuid,
    pub actor_id: Uuid,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub accept_waitlist: bool,
    pub invite_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct RsvpView {
    pub event_id: Uuid,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub entered_waitlist: bool,
    pub seats_remaining: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct GuestRsvp {
    pub view: RsvpView,
    pub guest_user_id: Uuid,
    pub guest_token: Option<String>,
}

pub async fn submit(db: &Db, input: SubmitRsvp) -> Result<RsvpView, AppError> {
    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;
    let view = apply(&mut tx, &input).await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;
    crate::activity::record_rsvp(db, input.event_id, input.actor_id, view.status).await;
    Ok(view)
}

#[derive(Debug, Clone)]
pub struct GuestRsvpInput<'a> {
    pub code: &'a str,
    pub display_name: &'a str,
    pub existing_guest: Option<Uuid>,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub accept_waitlist: bool,
    pub session_ttl_days: i64,
}

pub async fn submit_as_guest(db: &Db, input: GuestRsvpInput<'_>) -> Result<GuestRsvp, AppError> {
    let GuestRsvpInput {
        code: raw_code,
        display_name,
        existing_guest,
        status,
        plus_ones,
        accept_waitlist,
        session_ttl_days,
    } = input;

    let code = gathr_domain::InviteCode::parse(raw_code).map_err(|_| AppError::InviteInvalid)?;

    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;

    let invite = invites::lock_by_code(&mut tx, code.as_str())
        .await?
        .ok_or(AppError::InviteInvalid)?;
    terms(&invite).guard_redeemable(OffsetDateTime::now_utc())?;

    let (guest_user_id, is_new_guest) = match existing_guest {
        Some(user_id) => (user_id, false),
        None => {
            if display_name.trim().is_empty() {
                return Err(AppError::Validation("a name is required".to_owned()));
            }
            let created = users::insert(&mut tx, display_name.trim(), None, true).await?;
            (created.id, true)
        }
    };

    let view = apply(
        &mut tx,
        &SubmitRsvp {
            event_id: invite.event_id,
            actor_id: guest_user_id,
            status,
            plus_ones,
            accept_waitlist,
            invite_id: Some(invite.id),
        },
    )
    .await?;

    let guest_token = if is_new_guest {
        invites::increment_uses(&mut tx, invite.id).await?;
        let token = random_token();
        tokens::insert_guest_session(
            &mut tx,
            guest_user_id,
            &hash_token(&token),
            Some(invite.id),
            OffsetDateTime::now_utc() + Duration::days(session_ttl_days),
        )
        .await?;
        Some(token)
    } else {
        None
    };

    tx.commit().await.map_err(DbError::from_sqlx)?;
    crate::activity::record_rsvp(db, invite.event_id, guest_user_id, view.status).await;

    Ok(GuestRsvp {
        view,
        guest_user_id,
        guest_token,
    })
}

