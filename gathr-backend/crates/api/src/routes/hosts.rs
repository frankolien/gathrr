use actix_web::{web, HttpResponse};
use gathr_application::hosts;
use uuid::Uuid;

use crate::dto::{AddHostRequest, HostListResponse};
use crate::error::ApiError;
use crate::extract::AuthUser;
use crate::state::AppState;

pub async fn list(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let roster = hosts::roster(&state.db, path.into_inner(), user.0).await?;
    Ok(HttpResponse::Ok().json(HostListResponse::from(roster)))
}

pub async fn add(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<AddHostRequest>,
) -> Result<HttpResponse, ApiError> {
    let roster = hosts::invite(&state.db, path.into_inner(), user.0, body.user_id).await?;
    Ok(HttpResponse::Created().json(HostListResponse::from(roster)))
}

pub async fn remove(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, ApiError> {
    let (event_id, target_id) = path.into_inner();
    hosts::stand_down(&state.db, event_id, user.0, target_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
