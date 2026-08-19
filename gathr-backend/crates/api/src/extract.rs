use actix_web::dev::Payload;
use actix_web::http::header::AUTHORIZATION;
use actix_web::{web, FromRequest, HttpRequest};
use gathr_application::{auth, AppError};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub const GUEST_COOKIE: &str = "gathr_guest";
pub const IDEMPOTENCY_HEADER: &str = "idempotency-key";

pub struct AuthUser(pub Uuid);

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(authenticate(req).map(AuthUser).map_err(ApiError))
    }
}

fn authenticate(req: &HttpRequest) -> Result<Uuid, AppError> {
    let state = req
        .app_data::<web::Data<AppState>>()
        .ok_or(AppError::Unauthenticated)?;

    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthenticated)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthenticated)?;

    auth::verify_access(&state.tokens, token.trim())
}

pub struct IdempotencyKey(pub String);

