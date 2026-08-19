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

pub struct JwksCache {
    url: String,
    ttl: Duration,
    client: reqwest::Client,
    cached: RwLock<Option<(JwkSet, Instant)>>,
}

impl JwksCache {
    pub fn new(url: impl Into<String>, ttl: Duration) -> Self {
        Self {
            url: url.into(),
            ttl,
            client: reqwest::Client::new(),
            cached: RwLock::new(None),
        }
    }

    pub async fn current(&self) -> Result<JwkSet, OidcError> {
        if let Some((keys, fetched_at)) = self.cached.read().await.as_ref() {
            if fetched_at.elapsed() < self.ttl {
                return Ok(keys.clone());
            }
        }

        let fetched: JwkSet = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|error| OidcError::KeysUnavailable(error.to_string()))?
            .error_for_status()
            .map_err(|error| OidcError::KeysUnavailable(error.to_string()))?
            .json()
            .await
            .map_err(|error| OidcError::KeysUnavailable(error.to_string()))?;

        *self.cached.write().await = Some((fetched.clone(), Instant::now()));
        Ok(fetched)
    }
}
