use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::OidcError;
use crate::jwks::JwkSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Apple,
    Google,
}

impl Provider {
    pub fn issuers(&self) -> &'static [&'static str] {
        match self {
            Provider::Apple => &["https://appleid.apple.com"],
            Provider::Google => &["https://accounts.google.com", "accounts.google.com"],
        }
    }

    pub fn expected_nonce(&self, raw: &str) -> String {
        match self {
            Provider::Apple => {
                let digest = Sha256::digest(raw.as_bytes());
                digest.iter().map(|byte| format!("{byte:02x}")).collect()
            }
            Provider::Google => raw.to_owned(),
        }
    }
}

const CLOCK_SKEW_ALLOWANCE_SECONDS: u64 = 30;

#[derive(Debug, Deserialize)]
struct IdentityClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    nonce: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VerifiedIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub fn verify(
    token: &str,
    keys: &JwkSet,
    provider: Provider,
    audiences: &[String],
    raw_nonce: Option<&str>,
) -> Result<VerifiedIdentity, OidcError> {
    let header =
        decode_header(token).map_err(|error| OidcError::InvalidToken(error.to_string()))?;
    let kid = header.kid.ok_or(OidcError::UnknownSigningKey)?;
    let key = keys.decoding_key(&kid)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(provider.issuers());
    validation.set_audience(audiences);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.leeway = CLOCK_SKEW_ALLOWANCE_SECONDS;

    let data = decode::<IdentityClaims>(token, &key, &validation)
        .map_err(|error| OidcError::InvalidToken(error.to_string()))?;

    if data.claims.sub.trim().is_empty() {
        return Err(OidcError::MissingSubject);
    }

    if let Some(raw) = raw_nonce {
        let expected = provider.expected_nonce(raw);
        if data.claims.nonce.as_deref() != Some(expected.as_str()) {
            return Err(OidcError::NonceMismatch);
        }
    }

    Ok(VerifiedIdentity {
        subject: data.claims.sub,
        email: data.claims.email,
        name: data.claims.name,
    })
}
