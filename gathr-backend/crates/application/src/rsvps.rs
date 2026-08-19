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

pub async fn promote(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
    guest_id: Uuid,
) -> Result<RsvpView, AppError> {
    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;

    let event = events::lock(&mut tx, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if event.host_id != actor_id {
        return Err(AppError::Forbidden);
    }

    let current = rsvps::find_in_tx(&mut tx, event_id, guest_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let seats_held = rsvps::seats_held_excluding(&mut tx, event_id, guest_id).await?;

    let outcome = rsvp::promote_from_waitlist(
        current.status,
        current.plus_ones,
        CapacityContext {
            capacity: event.capacity,
            seats_held_excluding_actor: seats_held,
            max_plus_ones: event.max_plus_ones,
        },
    )?;

    let record = rsvps::upsert(
        &mut tx,
        event_id,
        guest_id,
        outcome.status,
        outcome.plus_ones,
        None,
    )
    .await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;

    Ok(RsvpView {
        event_id,
        status: record.status,
        plus_ones: record.plus_ones,
        entered_waitlist: false,
        seats_remaining: event
            .capacity
            .map(|capacity| (capacity - seats_held - 1 - record.plus_ones).max(0)),
    })
}

pub async fn guest_list(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
) -> Result<Vec<GuestRecord>, AppError> {
    let event = events::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let is_member =
        event.host_id == actor_id || rsvps::find(db, event_id, actor_id).await?.is_some();
    if !is_member {
        return Err(AppError::Forbidden);
    }
    Ok(rsvps::list_guests(db, event_id).await?)
}

async fn apply(tx: &mut Tx<'_>, input: &SubmitRsvp) -> Result<RsvpView, AppError> {
    let event = events::lock(tx, input.event_id)
        .await?
        .ok_or(AppError::NotFound)?;

    event::guard_accepts_rsvps(observed_status(&event, OffsetDateTime::now_utc()))?;

    let current = rsvps::find_in_tx(tx, input.event_id, input.actor_id).await?;
    let seats_held = rsvps::seats_held_excluding(tx, input.event_id, input.actor_id).await?;

    let outcome = rsvp::submit(
        current.map(|record| record.status),
        RsvpRequest {
            status: input.status,
            plus_ones: input.plus_ones,
            accept_waitlist: input.accept_waitlist,
        },
        CapacityContext {
            capacity: event.capacity,
            seats_held_excluding_actor: seats_held,
            max_plus_ones: event.max_plus_ones,
        },
    )?;

    let record = rsvps::upsert(
        tx,
        input.event_id,
        input.actor_id,
        outcome.status,
        outcome.plus_ones,
        input.invite_id,
    )
    .await?;

    let consumed = if record.status.holds_seats() {
        1 + record.plus_ones
    } else {
        0
    };

    Ok(RsvpView {
        event_id: input.event_id,
        status: record.status,
        plus_ones: record.plus_ones,
        entered_waitlist: outcome.entered_waitlist,
        seats_remaining: event
            .capacity
            .map(|capacity| (capacity - seats_held - consumed).max(0)),
    })
}
