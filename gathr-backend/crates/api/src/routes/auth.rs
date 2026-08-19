use actix_web::{web, HttpResponse};
use gathr_application::{auth, media, otp, AppError};
use gathr_infra_db::users;
use gathr_infra_oidc::{self as oidc, Provider};

use crate::dto::{
    ClaimRequest, ClaimResponse, DevSignInRequest, MeResponse, OAuthSignInRequest,
    OtpChallengeResponse, OtpRequest, OtpVerifyRequest, RefreshRequest, TokenResponse,
    UpdateProfileRequest,
};
use crate::error::ApiError;
use crate::extract::AuthUser;
use crate::state::AppState;

pub async fn sign_in_dev(
    state: web::Data<AppState>,
    body: web::Json<DevSignInRequest>,
) -> Result<HttpResponse, ApiError> {
    if !state.config.allow_dev_auth {
        return Err(ApiError(AppError::Forbidden));
    }

    let (user_id, pair) = auth::sign_in_dev(
        &state.db,
        &state.tokens,
        &body.display_name,
        body.phone.as_deref(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        user_id,
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        expires_in_seconds: pair.expires_in_seconds,
    }))
}

pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ApiError> {
    let pair = auth::rotate(&state.db, &state.tokens, &body.refresh_token).await?;
    let user_id = auth::verify_access(&state.tokens, &pair.access_token)?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        user_id,
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        expires_in_seconds: pair.expires_in_seconds,
    }))
}

pub async fn claim(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<ClaimRequest>,
) -> Result<HttpResponse, ApiError> {
    let claimed = gathr_application::claim::claim(&state.db, user.0, &body.guest_token).await?;

    Ok(HttpResponse::Ok().json(ClaimResponse {
        claimed: true,
        rsvps_moved: claimed.rsvps_moved,
        messages_moved: claimed.messages_moved,
    }))
}

pub async fn me(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, ApiError> {
    let record = users::find(&state.db, user.0)
        .await
        .map_err(AppError::from)?
        .ok_or(ApiError(AppError::NotFound))?;

    let avatar_url = media::avatar_url(&state.db, state.cloudinary.as_ref(), user.0).await?;

    Ok(HttpResponse::Ok().json(MeResponse {
        id: record.id,
        display_name: record.display_name,
        is_guest: record.is_guest,
        bio: record.bio,
        avatar_url,
    }))
}

fn provider_for(name: &str) -> Result<Provider, ApiError> {
    match name {
        "apple" => Ok(Provider::Apple),
        "google" => Ok(Provider::Google),
        _ => Err(ApiError(AppError::Validation(format!(
            "{name} is not a supported sign-in provider"
        )))),
    }
}

pub async fn sign_in_with_provider(
    state: web::Data<AppState>,
    body: web::Json<OAuthSignInRequest>,
) -> Result<HttpResponse, ApiError> {
    let provider = provider_for(&body.provider)?;
    let (keys, audiences) = match provider {
        Provider::Apple => (&state.apple_keys, &state.config.apple_audiences),
        Provider::Google => (&state.google_keys, &state.config.google_audiences),
    };

    if audiences.is_empty() {
        return Err(ApiError(AppError::ProviderUnavailable));
    }

    let key_set = keys
        .current()
        .await
        .map_err(|_| ApiError(AppError::ProviderUnavailable))?;

    let identity = oidc::verify(
        &body.id_token,
        &key_set,
        provider,
        audiences,
        body.nonce.as_deref(),
    )
    .map_err(|error| ApiError(AppError::IdentityRejected(error.to_string())))?;

    let (user_id, pair) = auth::sign_in_with_identity(
        &state.db,
        &state.tokens,
        auth::IdentitySignIn {
            provider: &body.provider,
            subject: &identity.subject,
            email: identity.email.as_deref(),
            display_name: body.display_name.as_deref().or(identity.name.as_deref()),
        },
    )
    .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        user_id,
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        expires_in_seconds: pair.expires_in_seconds,
    }))
}

pub async fn update_profile(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, ApiError> {
    auth::update_profile(
        &state.db,
        user.0,
        auth::ProfileUpdate {
            display_name: body.display_name.as_deref(),
            bio: body.bio.as_ref().map(|bio| bio.as_deref()),
            avatar_media_id: body.avatar_media_id,
        },
    )
    .await?;

    me(state, user).await
}

pub async fn request_code(
    state: web::Data<AppState>,
    body: web::Json<OtpRequest>,
) -> Result<HttpResponse, ApiError> {
    let channel = otp::Channel::parse(&body.channel)?;
    let challenge = otp::request(
        &state.db,
        channel,
        &body.destination,
        otp::Delivery {
            email: state.email.as_ref(),
            reveal_instead_of_sending: state.config.allow_dev_auth,
        },
    )
    .await?;

    Ok(HttpResponse::Accepted().json(OtpChallengeResponse {
        destination: challenge.destination,
        expires_in_seconds: challenge.expires_in_seconds,
        development_code: challenge.code_for_development,
    }))
}

pub async fn verify_code(
    state: web::Data<AppState>,
    body: web::Json<OtpVerifyRequest>,
) -> Result<HttpResponse, ApiError> {
    let channel = otp::Channel::parse(&body.channel)?;
    let (user_id, pair) = otp::verify(
        &state.db,
        &state.tokens,
        channel,
        &body.destination,
        &body.code,
    )
    .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        user_id,
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        expires_in_seconds: pair.expires_in_seconds,
    }))
}

pub async fn logout(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, ApiError> {
    auth::revoke_all_sessions(&state.db, user.0).await?;
    Ok(HttpResponse::NoContent().finish())
}
