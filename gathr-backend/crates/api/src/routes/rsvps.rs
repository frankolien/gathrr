use actix_web::{web, HttpResponse};
use gathr_application::rsvps::{self, SubmitRsvp};
use gathr_domain::RsvpStatus;
use uuid::Uuid;

use crate::dto::{GuestListResponse, GuestResponse, RsvpRequestBody, RsvpResponse};
use crate::error::ApiError;
use crate::extract::{AuthUser, IdempotencyKey};
use crate::idempotency;
use crate::state::AppState;

pub async fn submit(
    state: web::Data<AppState>,
    user: AuthUser,
    key: IdempotencyKey,
    path: web::Path<Uuid>,
    body: web::Json<RsvpRequestBody>,
) -> Result<HttpResponse, ApiError> {
    let event_id = path.into_inner();
    let fingerprint =
        idempotency::fingerprint(&(event_id, body.status, body.plus_ones, body.accept_waitlist));
    if let Some(replayed) = idempotency::replay(&state, &key.0, user.0, &fingerprint).await? {
        return Ok(replayed);
    }

    let view = rsvps::submit(
        &state.db,
        SubmitRsvp {
            event_id,
            actor_id: user.0,
            status: body.status,
            plus_ones: body.plus_ones,
            accept_waitlist: body.accept_waitlist,
            invite_id: None,
        },
    )
    .await?;

    let payload = serde_json::to_value(RsvpResponse::from(view)).unwrap_or_default();
    idempotency::record(&state, &key.0, user.0, &fingerprint, 200, &payload).await?;
    Ok(HttpResponse::Ok().json(payload))
}

pub async fn guests(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let records = rsvps::guest_list(&state.db, path.into_inner(), user.0).await?;

    let going = records
        .iter()
        .filter(|record| record.status == RsvpStatus::Going)
        .count();
    let seats_taken: i32 = records
        .iter()
        .filter(|record| record.status == RsvpStatus::Going)
        .map(|record| 1 + record.plus_ones)
        .sum();

    Ok(HttpResponse::Ok().json(GuestListResponse {
        going: i32::try_from(going).unwrap_or(i32::MAX),
        seats_taken,
        guests: records.into_iter().map(GuestResponse::from).collect(),
    }))
}

