use actix_web::{web, HttpResponse};
use gathr_application::invites;
use uuid::Uuid;

use crate::dto::{CreateInviteRequest, InviteResponse, PublicInviteResponse};
use crate::error::ApiError;
use crate::extract::AuthUser;
use crate::state::AppState;

pub async fn create(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateInviteRequest>,
) -> Result<HttpResponse, ApiError> {
    let record = invites::create(
        &state.db,
        path.into_inner(),
        user.0,
        body.max_uses,
        body.expires_at,
    )
    .await?;

    Ok(HttpResponse::Created().json(InviteResponse::new(record, &state.config.public_base_url)))
}

pub async fn list(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let records = invites::list(&state.db, path.into_inner(), user.0).await?;
    let responses: Vec<InviteResponse> = records
        .into_iter()
        .map(|record| InviteResponse::new(record, &state.config.public_base_url))
        .collect();
    Ok(HttpResponse::Ok().json(responses))
}

