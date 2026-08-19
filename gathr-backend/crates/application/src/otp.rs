use rand::Rng;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use gathr_infra_db::{otp, users, Db};
use gathr_infra_email::{verification_message, Resend};

use crate::auth::{self, TokenPair, TokenSettings};
use crate::error::AppError;

const CODE_LENGTH: u32 = 6;
const CODE_TTL_MINUTES: i64 = 10;
const MAX_ATTEMPTS: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Email,
}

