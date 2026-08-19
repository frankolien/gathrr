use std::time::{Duration, Instant};

use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::error::OidcError;

#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

impl JwkSet {
    pub fn decoding_key(&self, kid: &str) -> Result<DecodingKey, OidcError> {
        let jwk = self
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or(OidcError::UnknownSigningKey)?;

        DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|error| OidcError::MalformedSigningKey(error.to_string()))
    }
}

