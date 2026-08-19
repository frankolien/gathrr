use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use gathr_application::{messages, AppError};
use gathr_domain::PostingRight;
use uuid::Uuid;

use crate::dto::{MessageListResponse, MessageResponse, PostMessageRequest};
use crate::error::ApiError;
use crate::extract::{AuthUser, IdempotencyKey};
use crate::idempotency;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct MessagePage {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn post(
    state: web::Data<AppState>,
    user: AuthUser,
    key: IdempotencyKey,
    path: web::Path<Uuid>,
    body: web::Json<PostMessageRequest>,
) -> Result<HttpResponse, ApiError> {
    let event_id = path.into_inner();
    let fingerprint = idempotency::fingerprint(&(event_id, &body.body));
    if let Some(replayed) = idempotency::replay(&state, &key.0, user.0, &fingerprint).await? {
        return Ok(replayed);
    }

    let view = messages::post(
        &state.db,
        event_id,
        user.0,
        &body.body,
        PostingRight::Participant,
    )
    .await?;

    let response = MessageResponse::from(view);
    let payload = serde_json::to_value(&response).unwrap_or_default();
    state.hub.broadcast(event_id, &payload.to_string()).await;

    idempotency::record(&state, &key.0, user.0, &fingerprint, 201, &payload).await?;
    Ok(HttpResponse::Created().json(payload))
}

