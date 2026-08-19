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

pub async fn cancel(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let record = events::cancel(&state.db, path.into_inner(), user.0).await?;
    Ok(HttpResponse::Ok().json(EventSummary::from(record)))
}

pub async fn feed(
    state: web::Data<AppState>,
    user: AuthUser,
    query: web::Query<FeedQuery>,
) -> Result<HttpResponse, ApiError> {
    let records = events::feed(&state.db, user.0, &query.filter).await?;
    let summaries: Vec<EventSummary> = records.into_iter().map(EventSummary::from).collect();
    Ok(HttpResponse::Ok().json(summaries))
}

pub async fn edit(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<EditEventRequest>,
) -> Result<HttpResponse, ApiError> {
    let body = body.into_inner();
    let summary = events::edit(
        &state.db,
        path.into_inner(),
        user.0,
        events::EditEvent {
            title: body.title,
            category: body.category,
            description: body.description,
            location_name: body.location_name,
            starts_at: body.starts_at,
            ends_at: body.ends_at,
            timezone: body.timezone,
            capacity: body.capacity,
            max_plus_ones: body.max_plus_ones,
        },
    )
    .await?;

    Ok(HttpResponse::Ok().json(EventSummary::from(summary)))
}

pub async fn remove_guest(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, ApiError> {
    let (event_id, guest_id) = path.into_inner();
    events::remove_guest(&state.db, event_id, user.0, guest_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
