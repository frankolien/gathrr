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

pub async fn list(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<MessagePage>,
) -> Result<HttpResponse, ApiError> {
    let event_id = path.into_inner();
    let views = messages::page(
        &state.db,
        event_id,
        user.0,
        query.after_seq.unwrap_or(0),
        query.limit,
    )
    .await?;

    let latest_seq = views
        .last()
        .map(|view| view.seq)
        .unwrap_or(messages::latest_seq(&state.db, event_id, user.0).await?);

    Ok(HttpResponse::Ok().json(MessageListResponse {
        latest_seq,
        messages: views.into_iter().map(MessageResponse::from).collect(),
    }))
}

pub async fn stream(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
    request: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, ApiError> {
    let event_id = path.into_inner();
    messages::authorize_read(&state.db, event_id, user.0).await?;

    let (response, session, mut incoming) = actix_ws::handle(&request, stream).map_err(|_| {
        ApiError(AppError::Validation(
            "this is not a websocket request".to_owned(),
        ))
    })?;

    let hub = state.hub.clone();
    let subscriber_id = hub.join(event_id, session.clone()).await;

    actix_web::rt::spawn(async move {
        while let Some(Ok(message)) = incoming.next().await {
            if let actix_ws::Message::Close(_) = message {
                break;
            }
        }
        hub.leave(event_id, subscriber_id).await;
        let _ = session.close(None).await;
    });

    Ok(response)
}
