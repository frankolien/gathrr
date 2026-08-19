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

pub async fn blocks(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(BlockListResponse {
        blocked_user_ids: moderation::blocked(&state.db, user.0).await?,
    }))
}

pub async fn export(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, ApiError> {
    let export = account::export(&state.db, user.0).await?;
    let avatar_url = media::avatar_url(&state.db, state.cloudinary.as_ref(), user.0).await?;

    Ok(HttpResponse::Ok()
        .insert_header((
            "content-disposition",
            "attachment; filename=\"gathr-export.json\"",
        ))
        .json(ExportResponse {
            exported_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .unwrap_or_default(),
            account: MeResponse {
                id: export.account.id,
                display_name: export.account.display_name,
                is_guest: export.account.is_guest,
                bio: export.account.bio,
                avatar_url,
            },
            hosted_events: export
                .hosted_events
                .into_iter()
                .map(|event| ExportedEventResponse {
                    id: event.id,
                    title: event.title,
                    starts_at: event.starts_at,
                    status: event.status,
                    created_at: event.created_at,
                })
                .collect(),
            rsvps: export
                .rsvps
                .into_iter()
                .map(|rsvp| ExportedRsvpResponse {
                    event_id: rsvp.event_id,
                    event_title: rsvp.event_title,
                    status: rsvp.status,
                    plus_ones: rsvp.plus_ones,
                    created_at: rsvp.created_at,
                })
                .collect(),
            messages: export
                .messages
                .into_iter()
                .map(|message| ExportedMessageResponse {
                    event_id: message.event_id,
                    event_title: message.event_title,
                    seq: message.seq,
                    body: message.body,
                    created_at: message.created_at,
                })
                .collect(),
            identities: export
                .identities
                .into_iter()
                .map(|identity| ExportedIdentityResponse {
                    provider: identity.provider,
                    created_at: identity.created_at,
                })
                .collect(),
            blocked_user_ids: export.blocked_user_ids,
        }))
}

pub async fn erase(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, ApiError> {
    let erasure = account::erase(&state.db, user.0).await?;

    Ok(HttpResponse::Ok().json(ErasureResponse {
        deleted: true,
        events_handed_over: erasure.events_handed_over,
        events_cancelled: erasure.events_cancelled,
        messages_redacted: erasure.messages_redacted,
    }))
}
