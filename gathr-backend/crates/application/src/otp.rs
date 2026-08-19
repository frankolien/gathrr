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

async fn deliver(
    channel: Channel,
    destination: &str,
    code: &str,
    email: Option<&Resend>,
) -> Result<(), AppError> {
    match channel {
        Channel::Email => {
            let sender = email.ok_or(AppError::ProviderUnavailable)?;
            let (subject, body) = verification_message(code);
            sender
                .send(destination, &subject, &body)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "could not send a verification code");
                    AppError::DeliveryFailed(error.to_string())
                })
        }
    }
}

pub async fn verify(
    db: &Db,
    settings: &TokenSettings,
    channel: Channel,
    destination: &str,
    code: &str,
) -> Result<(Uuid, TokenPair), AppError> {
    let destination = channel.normalize(destination)?;
    let challenge = otp::find_pending(db, channel.name(), &destination)
        .await?
        .ok_or(AppError::OtpInvalid)?;

    if challenge.attempts >= MAX_ATTEMPTS {
        otp::consume(db, challenge.id).await?;
        return Err(AppError::OtpAttemptsExceeded);
    }
    if challenge.expires_at <= OffsetDateTime::now_utc() {
        otp::consume(db, challenge.id).await?;
        return Err(AppError::OtpInvalid);
    }
    if auth::hash_token(code.trim()) != challenge.code_hash {
        otp::record_failed_attempt(db, challenge.id).await?;
        return Err(AppError::OtpInvalid);
    }

    otp::consume(db, challenge.id).await?;
    let user = find_or_create(db, channel, &destination).await?;
    let pair = auth::issue_pair(db, settings, user, Uuid::new_v4()).await?;
    Ok((user, pair))
}

async fn find_or_create(db: &Db, channel: Channel, destination: &str) -> Result<Uuid, AppError> {
    let existing = match channel {
        Channel::Email => users::find_claimed_by_email(db, destination).await?,
    };

    if let Some(user) = existing {
        return Ok(user.id);
    }

    let (email, phone) = match channel {
        Channel::Email => (Some(destination), None),
    };

    let mut tx = db
        .begin()
        .await
        .map_err(gathr_infra_db::DbError::from_sqlx)?;
    let created =
        users::insert_verified(&mut tx, placeholder_name(destination), email, phone).await?;
    tx.commit()
        .await
        .map_err(gathr_infra_db::DbError::from_sqlx)?;

    Ok(created.id)
}

fn placeholder_name(destination: &str) -> &str {
    destination.split('@').next().unwrap_or(destination)
}

