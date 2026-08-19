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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_alphabet_excludes_the_ambiguous_letters() {
        for excluded in [b'I', b'L', b'O', b'U'] {
            assert!(!ALPHABET.contains(&excluded));
        }
        assert_eq!(ALPHABET.len(), 32);
    }

    #[test]
    fn every_byte_maps_into_the_alphabet_without_modulo_bias() {
        let mut counts = [0usize; 32];
        for byte in 0..=u8::MAX {
            let index = usize::from(byte % 32);
            counts[index] += 1;
        }
        assert!(counts.iter().all(|count| *count == 8));
    }

    #[test]
    fn generated_codes_are_the_specified_length() {
        let code = InviteCode::from_entropy([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(code.as_str().len(), CODE_LENGTH);
        assert_eq!(code.as_str(), "0123456789");
    }

    #[test]
    fn ambiguous_input_is_normalized_the_crockford_way() {
        let typed_by_a_guest = InviteCode::parse("iloo-OO01ab").unwrap();
        assert_eq!(typed_by_a_guest.as_str(), "11000001AB");
    }

    #[test]
    fn separators_and_case_are_forgiven() {
        assert_eq!(
            InviteCode::parse("abcd-efgh jk").unwrap().as_str(),
            "ABCDEFGHJK"
        );
    }

    #[test]
    fn u_is_rejected_rather_than_normalized() {
        assert_eq!(
            InviteCode::parse("UUUUUUUUUU"),
            Err(DomainError::InviteCodeInvalid)
        );
    }

    #[test]
    fn wrong_length_input_is_rejected() {
        assert_eq!(
            InviteCode::parse("ABC"),
            Err(DomainError::InviteCodeInvalid)
        );
        assert_eq!(
            InviteCode::parse("ABCDEFGHJKM"),
            Err(DomainError::InviteCodeInvalid)
        );
    }

    #[test]
    fn a_generated_code_round_trips_through_parse() {
        let code = InviteCode::from_entropy([200, 31, 64, 7, 129, 255, 12, 90, 3, 44]);
        assert_eq!(InviteCode::parse(code.as_str()).unwrap(), code);
    }
}
