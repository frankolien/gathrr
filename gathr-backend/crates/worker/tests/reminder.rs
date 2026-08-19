use std::sync::{LazyLock, Mutex};

use gathr_application::notifications::{DAY_BEFORE, HOURS_BEFORE};
use gathr_infra_db::{devices, reminders, Db};
use gathr_infra_push::Notification;
use gathr_worker::{drain_once, Dispatcher};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

static A_SWEEP_CLAIMS_EVERY_DUE_JOB_IN_THE_DATABASE: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(Default::default);

async fn pool() -> Db {
    let _ = dotenvy::from_filename("../../.env");
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run these tests");
    gathr_infra_db::connect(&url, 8)
        .await
        .expect("postgres must be reachable")
}

#[derive(Default)]
struct Recorder {
    delivered: Mutex<Vec<Notification>>,
}

