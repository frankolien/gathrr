use actix_web::HttpResponse;
use gathr_application::AppError;
use gathr_infra_db::idempotency;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn fingerprint(payload: &impl Serialize) -> String {
    let canonical = serde_json::to_string(payload).unwrap_or_default();
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub async fn replay(
    state: &AppState,
    key: &str,
    user_id: Uuid,
    fingerprint: &str,
) -> Result<Option<HttpResponse>, ApiError> {
    let Some(stored) = idempotency::find(&state.db, key, user_id)
        .await
        .map_err(AppError::from)?
    else {
        return Ok(None);
    };

    if stored.request_hash != fingerprint {
        return Err(ApiError(AppError::IdempotencyConflict));
    }

    let status = stored
        .response_code
        .and_then(|code| u16::try_from(code).ok())
        .and_then(|code| actix_web::http::StatusCode::from_u16(code).ok())
        .unwrap_or(actix_web::http::StatusCode::OK);

    let body = stored.response_body.unwrap_or(Value::Null);
    Ok(Some(HttpResponse::build(status).json(body)))
}

pub async fn record(
    state: &AppState,
    key: &str,
    user_id: Uuid,
    fingerprint: &str,
    status: u16,
    body: &Value,
) -> Result<(), ApiError> {
    idempotency::save(
        &state.db,
        key,
        user_id,
        fingerprint,
        i32::from(status),
        body,
    )
    .await
    .map_err(AppError::from)?;
    Ok(())
}
