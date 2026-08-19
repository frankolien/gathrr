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

pub struct AppState {
    pub db: Db,
    pub config: Config,
    pub tokens: TokenSettings,
    pub templates: Environment<'static>,
    pub hub: EventHub,
    pub cloudinary: Option<Cloudinary>,
    pub email: Option<Resend>,
    pub limiter: Arc<RateLimiter>,
    pub apple_keys: JwksCache,
    pub google_keys: JwksCache,
}

impl AppState {
    pub fn new(db: Db, config: Config) -> Self {
        let tokens = TokenSettings {
            secret: config.jwt_secret.clone(),
            access_ttl_minutes: config.access_token_ttl_minutes,
            refresh_ttl_days: config.refresh_token_ttl_days,
        };

        let mut templates = Environment::new();
        templates
            .add_template("invite.html", include_str!("templates/invite.html"))
            .expect("the bundled invite template must parse");
        templates
            .add_template("message.html", include_str!("templates/message.html"))
            .expect("the bundled message template must parse");

        let cloudinary = Cloudinary::new(
            config.cloudinary_cloud_name.clone(),
            config.cloudinary_api_key.clone(),
            config.cloudinary_api_secret.clone(),
        );

        let email = Resend::new(config.resend_api_key.clone(), config.email_from.clone());

        Self {
            db,
            config,
            tokens,
            templates,
            cloudinary,
            email,
            hub: EventHub::default(),
            limiter: Arc::new(RateLimiter::default()),
            apple_keys: JwksCache::new(APPLE_KEYS_URL, KEY_CACHE_TTL),
            google_keys: JwksCache::new(GOOGLE_KEYS_URL, KEY_CACHE_TTL),
        }
    }
}
