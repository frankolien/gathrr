use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

pub const CODE_LENGTH: usize = 10;

const ALPHABET: [u8; 32] = *b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InviteCode(String);

