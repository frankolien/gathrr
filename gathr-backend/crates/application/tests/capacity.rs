use std::sync::Arc;

use gathr_application::auth::TokenSettings;
use gathr_application::events::{self, CreateEvent};
use gathr_application::rsvps::{self, SubmitRsvp};
use gathr_application::{auth, AppError};
use gathr_domain::{Category, DomainError, RsvpStatus};
use gathr_infra_db::Db;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

async fn pool() -> Db {
    let _ = dotenvy::from_filename("../../.env");
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run these tests");
    gathr_infra_db::connect(&url, 32)
        .await
        .expect("postgres must be reachable")
}

fn settings() -> TokenSettings {
    TokenSettings {
        secret: "integration-test-secret".to_owned(),
        access_ttl_minutes: 15,
        refresh_ttl_days: 60,
    }
}

async fn user(db: &Db, name: &str) -> Uuid {
    auth::sign_in_dev(db, &settings(), name, None)
        .await
        .expect("a user must be created")
        .0
}

