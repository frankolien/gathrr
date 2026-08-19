use thiserror::Error;

#[derive(Debug, Error)]
pub enum OidcError {
    #[error("the provider's signing keys could not be fetched: {0}")]
    KeysUnavailable(String),
    #[error("the token header names no known signing key")]
    UnknownSigningKey,
    #[error("the signing key is not usable: {0}")]
    MalformedSigningKey(String),
    #[error("the token is not valid: {0}")]
    InvalidToken(String),
    #[error("the token is missing a subject")]
    MissingSubject,
    #[error("the token was issued for a different sign-in attempt")]
    NonceMismatch,
}
