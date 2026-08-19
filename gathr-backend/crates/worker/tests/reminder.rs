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

impl Dispatcher for Recorder {
    async fn deliver(&self, notification: Notification) -> Result<(), String> {
        self.delivered.lock().unwrap().push(notification);
        Ok(())
    }
}

impl Recorder {
    fn for_event(&self, event: Uuid) -> Vec<String> {
        self.delivered
            .lock()
            .unwrap()
            .iter()
            .filter(|note| note.thread_id == event.to_string())
            .map(|note| note.body.clone())
            .collect()
    }
}

async fn seed_event(db: &Db, starts_at: OffsetDateTime) -> (Uuid, Uuid) {
    let host = sqlx::query_scalar!(
        r#"INSERT INTO users (display_name) VALUES ('Amara Chukwu') RETURNING id"#
    )
    .fetch_one(db)
    .await
    .unwrap();

    let event = sqlx::query_scalar!(
        r#"INSERT INTO events (host_id, title, starts_at, status)
           VALUES ($1, 'Group Therapy', $2, 'published') RETURNING id"#,
        host,
        starts_at
    )
    .fetch_one(db)
    .await
    .unwrap();

    (host, event)
}

