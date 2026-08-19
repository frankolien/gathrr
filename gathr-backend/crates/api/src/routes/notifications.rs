use actix_web::{web, HttpResponse};
use gathr_application::{activity, notifications};
use uuid::Uuid;

use crate::dto::{
    MarkReadRequest, MuteRequest, NotificationFeedQuery, NotificationFeedResponse,
    RegisterDeviceRequest, RegisterDeviceResponse, UnreadCountResponse,
};
use crate::error::ApiError;
use crate::extract::AuthUser;
use crate::state::AppState;

pub async fn register_device(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<RegisterDeviceRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = notifications::register_device(
        &state.db,
        user.0,
        &body.apns_token,
        body.environment.as_deref(),
    )
    .await?;

    Ok(HttpResponse::Created().json(RegisterDeviceResponse { id }))
}

pub async fn forget_device(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    notifications::forget_device(&state.db, user.0, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn mute(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<MuteRequest>,
) -> Result<HttpResponse, ApiError> {
    let event_id = path.into_inner();
    notifications::set_mute(&state.db, user.0, event_id, body.muted).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "muted": body.muted })))
}

pub async fn feed(
    state: web::Data<AppState>,
    user: AuthUser,
    query: web::Query<NotificationFeedQuery>,
) -> Result<HttpResponse, ApiError> {
    let page = activity::page(&state.db, user.0, query.before, query.limit).await?;

    Ok(HttpResponse::Ok().json(NotificationFeedResponse {
        unread: page.unread,
        notifications: page.entries.into_iter().map(Into::into).collect(),
    }))
}

pub async fn mark_read(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<MarkReadRequest>,
) -> Result<HttpResponse, ApiError> {
    let unread = activity::mark_read(&state.db, user.0, &body.ids).await?;
    Ok(HttpResponse::Ok().json(UnreadCountResponse { unread }))
}
