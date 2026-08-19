use std::time::Duration;

use gathr_application::auth::TokenSettings;
use gathr_common::Config;
use gathr_infra_db::Db;
use gathr_infra_email::Resend;
use gathr_infra_media::Cloudinary;
use gathr_infra_oidc::JwksCache;

use std::sync::Arc;

use crate::hub::EventHub;
use crate::ratelimit::RateLimiter;
use minijinja::Environment;

const APPLE_KEYS_URL: &str = "https://appleid.apple.com/auth/keys";
const GOOGLE_KEYS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const KEY_CACHE_TTL: Duration = Duration::from_secs(3_600);

