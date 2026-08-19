use rand::Rng;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use gathr_infra_db::{otp, users, Db};
use gathr_infra_email::{verification_message, Resend};

use crate::auth::{self, TokenPair, TokenSettings};
use crate::error::AppError;

const CODE_LENGTH: u32 = 6;
const CODE_TTL_MINUTES: i64 = 10;
const MAX_ATTEMPTS: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Email,
}

impl Channel {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "email" => Ok(Self::Email),
            other => Err(AppError::Validation(format!(
                "{other} is not a supported verification channel"
            ))),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Email => "email",
        }
    }

    fn normalize(&self, destination: &str) -> Result<String, AppError> {
        let trimmed = destination.trim();
        match self {
            Self::Email => {
                let lowered = trimmed.to_lowercase();
                let valid = lowered.split('@').count() == 2
                    && !lowered.starts_with('@')
                    && !lowered.ends_with('@')
                    && lowered.contains('.');
                valid
                    .then_some(lowered)
                    .ok_or_else(|| AppError::Validation("that email doesn't look right".to_owned()))
            }
        }
    }
}

pub struct Challenge {
    pub destination: String,
    pub expires_in_seconds: i64,
    pub code_for_development: Option<String>,
}

pub struct Delivery<'a> {
    pub email: Option<&'a Resend>,
    pub reveal_instead_of_sending: bool,
}

pub async fn request(
    db: &Db,
    channel: Channel,
    destination: &str,
    delivery: Delivery<'_>,
) -> Result<Challenge, AppError> {
    let destination = channel.normalize(destination)?;
    let code = generate_code();

    if !delivery.reveal_instead_of_sending {
        deliver(channel, &destination, &code, delivery.email).await?;
    }

    otp::supersede_pending(db, channel.name(), &destination).await?;
    otp::insert(
        db,
        channel.name(),
        &destination,
        &auth::hash_token(&code),
        OffsetDateTime::now_utc() + Duration::minutes(CODE_TTL_MINUTES),
    )
    .await?;

    Ok(Challenge {
        destination,
        expires_in_seconds: CODE_TTL_MINUTES * 60,
        code_for_development: delivery.reveal_instead_of_sending.then_some(code),
    })
}

