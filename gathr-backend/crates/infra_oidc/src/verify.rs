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

