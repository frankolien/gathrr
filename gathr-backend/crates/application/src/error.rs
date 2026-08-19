use gathr_domain::DomainError;
use gathr_infra_db::DbError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("this resource does not exist")]
    NotFound,

    #[error("only the host can do this")]
    Forbidden,

    #[error("authentication is required")]
    Unauthenticated,

    #[error("this session has been revoked, please sign in again")]
    TokenReuseDetected,

    #[error("this invite link isn't valid")]
    InviteInvalid,

    #[error("this key was already used with a different request")]
    IdempotencyConflict,

    #[error("{0}")]
    Validation(String),

    #[error("could not allocate a unique invite code")]
    CodeExhaustion,

    #[error("this sign-in could not be verified: {0}")]
    IdentityRejected(String),

    #[error("this sign-in provider is not available")]
    ProviderUnavailable,

    #[error("that code isn't right")]
    OtpInvalid,

    #[error("too many attempts, request a new code")]
    OtpAttemptsExceeded,

    #[error("we could not send your code: {0}")]
    DeliveryFailed(String),

    #[error(transparent)]
    Db(#[from] DbError),
}

