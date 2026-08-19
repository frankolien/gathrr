use std::env::{self, VarError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0} is not set")]
    Missing(&'static str),
    #[error("{key} is not valid: {reason}")]
    Invalid { key: &'static str, reason: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub public_base_url: String,
    pub jwt_secret: String,
    pub allow_dev_auth: bool,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub guest_session_ttl_days: i64,
    pub apple_audiences: Vec<String>,
    pub google_audiences: Vec<String>,
    pub cloudinary_cloud_name: String,
    pub cloudinary_api_key: String,
    pub cloudinary_api_secret: String,
    pub resend_api_key: String,
    pub email_from: String,
    pub apns_team_id: String,
    pub apns_key_id: String,
    pub apns_private_key: String,
    pub apns_topic: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            bind_addr: optional("GATHR_BIND_ADDR", "127.0.0.1:8080"),
            public_base_url: optional("GATHR_PUBLIC_BASE_URL", "http://localhost:8080"),
            jwt_secret: required("GATHR_JWT_SECRET")?,
            allow_dev_auth: flag("GATHR_ALLOW_DEV_AUTH")?,
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 60,
            guest_session_ttl_days: 90,
            apple_audiences: list("GATHR_APPLE_AUDIENCES"),
            google_audiences: list("GATHR_GOOGLE_AUDIENCES"),
            cloudinary_cloud_name: optional("CLOUDINARY_CLOUD_NAME", ""),
            cloudinary_api_key: optional("CLOUDINARY_API_KEY", ""),
            cloudinary_api_secret: optional("CLOUDINARY_API_SECRET", ""),
            resend_api_key: optional("RESEND_API_KEY", ""),
            email_from: optional("GATHR_EMAIL_FROM", ""),
            apns_team_id: optional("APNS_TEAM_ID", ""),
            apns_key_id: optional("APNS_KEY_ID", ""),
            apns_private_key: optional("APNS_PRIVATE_KEY", "").replace("\\n", "\n"),
            apns_topic: optional("APNS_TOPIC", "app.gathr.ios"),
        })
    }
}

