use actix_web::{web, HttpResponse};
use gathr_application::{account, media, moderation};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::{
    BlockListResponse, BlockRequest, ErasureResponse, ExportResponse, ExportedEventResponse,
    ExportedIdentityResponse, ExportedMessageResponse, ExportedRsvpResponse, MeResponse,
    ReportRequest, ReportResponse,
};
use crate::error::ApiError;
use crate::extract::AuthUser;
use crate::state::AppState;

pub async fn report(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<ReportRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = moderation::report(
        &state.db,
        moderation::NewReport {
            reporter_id: user.0,
            subject: &body.subject,
            subject_id: body.subject_id,
            reason: &body.reason,
            detail: body.detail.as_deref(),
        },
    )
    .await?;

    Ok(HttpResponse::Created().json(ReportResponse { id }))
}

pub async fn block(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<BlockRequest>,
) -> Result<HttpResponse, ApiError> {
    moderation::block(&state.db, user.0, body.user_id).await?;
    Ok(HttpResponse::Created().json(BlockListResponse {
        blocked_user_ids: moderation::blocked(&state.db, user.0).await?,
    }))
}

pub async fn unblock(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    moderation::unblock(&state.db, user.0, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

