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

#[derive(Debug, Clone)]
pub struct Notification {
    pub device_token: String,
    pub environment: String,
    pub title: String,
    pub body: String,
    pub thread_id: String,
}

#[derive(Serialize)]
struct Payload<'a> {
    aps: Aps<'a>,
    event_id: &'a str,
}

#[derive(Serialize)]
struct Aps<'a> {
    alert: Alert<'a>,
    sound: &'a str,
    #[serde(rename = "thread-id")]
    thread_id: &'a str,
}

#[derive(Serialize)]
struct Alert<'a> {
    title: &'a str,
    body: &'a str,
}

#[derive(Serialize)]
struct ProviderClaims<'a> {
    iss: &'a str,
    iat: i64,
}

struct CachedToken {
    value: String,
    issued_at: OffsetDateTime,
}

pub struct Apns {
    team_id: String,
    key_id: String,
    key: EncodingKey,
    topic: String,
    client: reqwest::Client,
    token: Mutex<Option<CachedToken>>,
}

impl Apns {
    pub fn new(
        team_id: String,
        key_id: String,
        private_key_pem: &str,
        topic: String,
    ) -> Option<Result<Self, PushError>> {
        let configured = [&team_id, &key_id, &topic]
            .iter()
            .all(|value| !value.trim().is_empty())
            && !private_key_pem.trim().is_empty();

        if !configured {
            return None;
        }

        Some(
            EncodingKey::from_ec_pem(private_key_pem.as_bytes())
                .map_err(|error| PushError::UnusableKey(error.to_string()))
                .map(|key| Self {
                    team_id,
                    key_id,
                    key,
                    topic,
                    client: reqwest::Client::new(),
                    token: Mutex::new(None),
                }),
        )
    }

    pub async fn send(&self, notification: &Notification) -> Result<(), PushError> {
        let host = if notification.environment == "production" {
            PRODUCTION_HOST
        } else {
            SANDBOX_HOST
        };

        let response = self
            .client
            .post(format!("{host}/3/device/{}", notification.device_token))
            .header(
                "authorization",
                format!("bearer {}", self.provider_token()?),
            )
            .header("apns-topic", &self.topic)
            .header("apns-push-type", "alert")
            .json(&Payload {
                aps: Aps {
                    alert: Alert {
                        title: &notification.title,
                        body: &notification.body,
                    },
                    sound: "default",
                    thread_id: &notification.thread_id,
                },
                event_id: &notification.thread_id,
            })
            .send()
            .await
            .map_err(|error| PushError::Unreachable(error.to_string()))?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let reason = response
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|body| {
                body.get("reason")
                    .and_then(|r| r.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| status.to_string());

        Err(PushError::Rejected {
            expired: matches!(reason.as_str(), "BadDeviceToken" | "Unregistered"),
            reason,
        })
    }

    fn provider_token(&self) -> Result<String, PushError> {
        let mut cached = self.token.lock().unwrap_or_else(|poisoned| {
            self.token.clear_poison();
            poisoned.into_inner()
        });

        let now = OffsetDateTime::now_utc();
        if let Some(token) = cached.as_ref() {
            if now - token.issued_at < Duration::minutes(TOKEN_LIFETIME_MINUTES) {
                return Ok(token.value.clone());
            }
        }

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        let value = jsonwebtoken::encode(
            &header,
            &ProviderClaims {
                iss: &self.team_id,
                iat: now.unix_timestamp(),
            },
            &self.key,
        )
        .map_err(|error| PushError::UnusableKey(error.to_string()))?;

        *cached = Some(CachedToken {
            value: value.clone(),
            issued_at: now,
        });
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_stays_unconfigured_until_every_piece_is_present() {
        assert!(Apns::new(String::new(), "K".into(), "pem", "app.gathr.ios".into()).is_none());
        assert!(Apns::new("T".into(), String::new(), "pem", "app.gathr.ios".into()).is_none());
        assert!(Apns::new("T".into(), "K".into(), "   ", "app.gathr.ios".into()).is_none());
        assert!(Apns::new("T".into(), "K".into(), "pem", String::new()).is_none());
    }

    #[test]
    fn a_malformed_key_is_reported_rather_than_panicking_at_send_time() {
        let built = Apns::new(
            "TEAMID1234".into(),
            "KEYID12345".into(),
            "not a pem at all",
            "app.gathr.ios".into(),
        );
        assert!(matches!(built, Some(Err(PushError::UnusableKey(_)))));
    }
}
