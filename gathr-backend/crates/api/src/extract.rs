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

