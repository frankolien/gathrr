use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

pub const CODE_LENGTH: usize = 10;

const ALPHABET: [u8; 32] = *b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InviteCode(String);

impl InviteCode {
    pub fn from_entropy(bytes: [u8; CODE_LENGTH]) -> Self {
        let symbols = bytes
            .iter()
            .map(|byte| ALPHABET[usize::from(byte % 32)] as char)
            .collect();
        Self(symbols)
    }

    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let normalized: String = input
            .chars()
            .filter(|character| !matches!(character, '-' | ' ' | '_'))
            .map(normalize_symbol)
            .collect::<Result<_, _>>()?;

        if normalized.len() != CODE_LENGTH {
            return Err(DomainError::InviteCodeInvalid);
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn normalize_symbol(character: char) -> Result<char, DomainError> {
    let upper = character.to_ascii_uppercase();
    let mapped = match upper {
        'I' | 'L' => '1',
        'O' => '0',
        other => other,
    };
    if ALPHABET.contains(&(mapped as u8)) {
        Ok(mapped)
    } else {
        Err(DomainError::InviteCodeInvalid)
    }
}

impl fmt::Display for InviteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

