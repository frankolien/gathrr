use actix_web::{web, HttpResponse};
use gathr_application::events::{self, CreateEvent};
use gathr_domain::{Category, DEFAULT_MAX_PLUS_ONES};
use uuid::Uuid;

use crate::dto::{
    CreateEventRequest, EditEventRequest, EventDetailResponse, EventSummary, FeedQuery,
};
use crate::error::ApiError;
use crate::extract::{AuthUser, IdempotencyKey};
use crate::idempotency;
use crate::state::AppState;

pub async fn create(
    state: web::Data<AppState>,
    user: AuthUser,
    key: IdempotencyKey,
    body: web::Json<CreateEventRequest>,
) -> Result<HttpResponse, ApiError> {
    let fingerprint = idempotency::fingerprint(&body.0);
    if let Some(replayed) = idempotency::replay(&state, &key.0, user.0, &fingerprint).await? {
        return Ok(replayed);
    }

    let record = events::create(
        &state.db,
        CreateEvent {
            host_id: user.0,
            title: body.title.clone(),
            category: body
                .category
                .as_deref()
                .map(Category::parse_or_other)
                .unwrap_or(Category::Other),
            description: body.description.clone(),
            location_name: body.location_name.clone(),
            starts_at: body.starts_at,
            ends_at: body.ends_at,
            timezone: body
                .timezone
                .clone()
                .unwrap_or_else(|| "Africa/Lagos".to_owned()),
            capacity: body.capacity,
            max_plus_ones: body.max_plus_ones.unwrap_or(DEFAULT_MAX_PLUS_ONES),
            publish_now: body.publish_now,
        },
    )
    .await?;

    let payload = serde_json::to_value(EventSummary::from(record)).unwrap_or_default();
    idempotency::record(&state, &key.0, user.0, &fingerprint, 201, &payload).await?;
    Ok(HttpResponse::Created().json(payload))
}

pub async fn detail(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let detail = events::detail(&state.db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(EventDetailResponse::from(detail)))
}

pub async fn publish(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let record = events::publish(&state.db, path.into_inner(), user.0).await?;
    Ok(HttpResponse::Ok().json(EventSummary::from(record)))
}

