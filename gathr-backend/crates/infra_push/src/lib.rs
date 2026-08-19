use std::sync::Mutex;

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Serialize;
use thiserror::Error;
use time::{Duration, OffsetDateTime};

const PRODUCTION_HOST: &str = "https://api.push.apple.com";
const SANDBOX_HOST: &str = "https://api.sandbox.push.apple.com";
const TOKEN_LIFETIME_MINUTES: i64 = 50;

#[derive(Debug, Error)]
pub enum PushError {
    #[error("the apns signing key is not usable: {0}")]
    UnusableKey(String),
    #[error("apns could not be reached: {0}")]
    Unreachable(String),
    #[error("apns rejected this notification: {reason}")]
    Rejected { reason: String, expired: bool },
}

