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

pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn issue_access(settings: &TokenSettings, user_id: Uuid) -> Result<String, AppError> {
    let expiry = OffsetDateTime::now_utc() + Duration::minutes(settings.access_ttl_minutes);
    let claims = Claims {
        sub: user_id.to_string(),
        jti: Uuid::new_v4().to_string(),
        iss: ISSUER.to_owned(),
        aud: AUDIENCE.to_owned(),
        exp: expiry.unix_timestamp(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(settings.secret.as_bytes()),
    )
    .map_err(|_| AppError::Unauthenticated)
}

pub fn verify_access(settings: &TokenSettings, token: &str) -> Result<Uuid, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(settings.secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthenticated)?;

    Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::Unauthenticated)
}

pub async fn issue_pair(
    db: &Db,
    settings: &TokenSettings,
    user_id: Uuid,
    family_id: Uuid,
) -> Result<TokenPair, AppError> {
    let refresh_token = random_token();
    let expires_at = OffsetDateTime::now_utc() + Duration::days(settings.refresh_ttl_days);

    tokens::insert_refresh(
        db,
        Uuid::new_v4(),
        family_id,
        user_id,
        &hash_token(&refresh_token),
        expires_at,
    )
    .await?;

    Ok(TokenPair {
        access_token: issue_access(settings, user_id)?,
        refresh_token,
        expires_in_seconds: settings.access_ttl_minutes * 60,
    })
}

pub async fn sign_in_dev(
    db: &Db,
    settings: &TokenSettings,
    display_name: &str,
    phone: Option<&str>,
) -> Result<(Uuid, TokenPair), AppError> {
    if display_name.trim().is_empty() {
        return Err(AppError::Validation(
            "a display name is required".to_owned(),
        ));
    }

    let existing = match phone {
        Some(phone) => users::find_claimed_by_phone(db, phone).await?,
        None => None,
    };

    let user = match existing {
        Some(user) => user,
        None => {
            let mut tx = db
                .begin()
                .await
                .map_err(gathr_infra_db::DbError::from_sqlx)?;
            let created = users::insert(&mut tx, display_name.trim(), phone, false).await?;
            tx.commit()
                .await
                .map_err(gathr_infra_db::DbError::from_sqlx)?;
            created
        }
    };

    let pair = issue_pair(db, settings, user.id, Uuid::new_v4()).await?;
    Ok((user.id, pair))
}

pub async fn rotate(
    db: &Db,
    settings: &TokenSettings,
    refresh_token: &str,
) -> Result<TokenPair, AppError> {
    let stored = tokens::find_refresh(db, &hash_token(refresh_token))
        .await?
        .ok_or(AppError::Unauthenticated)?;

    if stored.revoked_at.is_some() {
        return Err(AppError::TokenReuseDetected);
    }
    if stored.used_at.is_some() {
        tokens::revoke_family(db, stored.family_id).await?;
        return Err(AppError::TokenReuseDetected);
    }
    if stored.expires_at <= OffsetDateTime::now_utc() {
        return Err(AppError::Unauthenticated);
    }

    tokens::burn_refresh(db, stored.jti).await?;
    issue_pair(db, settings, stored.user_id, stored.family_id).await
}

pub async fn resolve_guest_session(db: &Db, token: &str) -> Result<Option<Uuid>, AppError> {
    Ok(tokens::find_guest_session(db, &hash_token(token)).await?)
}

pub struct IdentitySignIn<'a> {
    pub provider: &'a str,
    pub subject: &'a str,
    pub email: Option<&'a str>,
    pub display_name: Option<&'a str>,
}

fn resolve_display_name(input: &IdentitySignIn<'_>) -> Result<String, AppError> {
    let offered = input
        .display_name
        .map(str::trim)
        .filter(|name| !name.is_empty());

    if let Some(name) = offered {
        return Ok(name.to_owned());
    }

    input
        .email
        .and_then(|email| email.split('@').next())
        .map(str::trim)
        .filter(|local| !local.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AppError::Validation("a display name is required".to_owned()))
}

pub async fn sign_in_with_identity(
    db: &Db,
    settings: &TokenSettings,
    input: IdentitySignIn<'_>,
) -> Result<(Uuid, TokenPair), AppError> {
    if let Some(user) = identities::find_user(db, input.provider, input.subject).await? {
        let pair = issue_pair(db, settings, user.id, Uuid::new_v4()).await?;
        return Ok((user.id, pair));
    }

    let display_name = resolve_display_name(&input)?;

    let mut tx = db
        .begin()
        .await
        .map_err(gathr_infra_db::DbError::from_sqlx)?;
    let user = users::insert(&mut tx, &display_name, None, false).await?;
    identities::link(&mut tx, user.id, input.provider, input.subject, input.email).await?;
    tx.commit()
        .await
        .map_err(gathr_infra_db::DbError::from_sqlx)?;

    let pair = issue_pair(db, settings, user.id, Uuid::new_v4()).await?;
    Ok((user.id, pair))
}

pub struct ProfileUpdate<'a> {
    pub display_name: Option<&'a str>,
    pub bio: Option<Option<&'a str>>,
    pub avatar_media_id: Option<Uuid>,
}

pub const MAX_BIO_LENGTH: usize = 300;

pub async fn update_profile(
    db: &Db,
    user_id: Uuid,
    update: ProfileUpdate<'_>,
) -> Result<(), AppError> {
    if let Some(display_name) = update.display_name {
        if display_name.trim().is_empty() {
            return Err(AppError::Validation(
                "a display name is required".to_owned(),
            ));
        }
    }
    if let Some(Some(bio)) = update.bio {
        if bio.chars().count() > MAX_BIO_LENGTH {
            return Err(AppError::Validation(format!(
                "a bio cannot be longer than {MAX_BIO_LENGTH} characters"
            )));
        }
    }

    if let Some(media_id) = update.avatar_media_id {
        gathr_infra_db::media::find_owned(db, media_id, user_id)
            .await?
            .ok_or(AppError::NotFound)?;
    }

    let trimmed_name = update.display_name.map(str::trim);
    let trimmed_bio = update
        .bio
        .map(|bio| bio.map(str::trim).filter(|value| !value.is_empty()));

    users::update_profile(
        db,
        user_id,
        users::ProfileEdit {
            display_name: trimmed_name,
            bio: trimmed_bio,
            avatar_media_id: update.avatar_media_id,
        },
    )
    .await?
    .map(|_| ())
    .ok_or(AppError::NotFound)
}

pub async fn revoke_all_sessions(db: &Db, user_id: Uuid) -> Result<(), AppError> {
    Ok(tokens::revoke_all_for_user(db, user_id).await?)
}
