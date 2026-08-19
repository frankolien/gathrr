use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use gathr_infra_db::{identities, tokens, users, Db};

use crate::error::AppError;

const ISSUER: &str = "gathr";
const AUDIENCE: &str = "gathr-ios";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub jti: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct TokenSettings {
    pub secret: String,
    pub access_ttl_minutes: i64,
    pub refresh_ttl_days: i64,
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_seconds: i64,
}

