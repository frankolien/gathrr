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

fn token() -> String {
    Uuid::new_v4().simple().to_string().repeat(2)
}

#[tokio::test]
async fn a_due_reminder_reaches_every_unmuted_device_of_the_people_going() {
    let _sweeps_run_one_at_a_time = A_SWEEP_CLAIMS_EVERY_DUE_JOB_IN_THE_DATABASE.lock().await;
    let db = pool().await;
    let now = OffsetDateTime::now_utc();
    let (host, event) = seed_event(&db, now + Duration::hours(3)).await;

    devices::upsert(&db, host, &token(), "sandbox")
        .await
        .unwrap();
    devices::upsert(&db, host, &token(), "sandbox")
        .await
        .unwrap();
    reminders::schedule(&db, event, HOURS_BEFORE, now - Duration::minutes(1))
        .await
        .unwrap();

    let recorder = Recorder::default();
    let sweep = drain_once(&db, &recorder, now).await.unwrap();

    assert!(sweep.claimed >= 1);
    let bodies = recorder.for_event(event);
    assert_eq!(
        bodies.len(),
        2,
        "both of the host's devices should be reached"
    );
    assert!(bodies[0].contains("2 hours"));
}

#[tokio::test]
async fn muting_an_event_silences_it_without_unregistering_the_device() {
    let _sweeps_run_one_at_a_time = A_SWEEP_CLAIMS_EVERY_DUE_JOB_IN_THE_DATABASE.lock().await;
    let db = pool().await;
    let now = OffsetDateTime::now_utc();
    let (host, event) = seed_event(&db, now + Duration::hours(3)).await;

    devices::upsert(&db, host, &token(), "sandbox")
        .await
        .unwrap();
    devices::set_mute(&db, host, event, true).await.unwrap();
    reminders::schedule(&db, event, HOURS_BEFORE, now - Duration::minutes(1))
        .await
        .unwrap();

    let recorder = Recorder::default();
    drain_once(&db, &recorder, now).await.unwrap();

    assert!(
        recorder.for_event(event).is_empty(),
        "a muted event sends nothing"
    );
}

#[tokio::test]
async fn a_reminder_is_claimed_once_so_two_workers_cannot_both_send_it() {
    let _sweeps_run_one_at_a_time = A_SWEEP_CLAIMS_EVERY_DUE_JOB_IN_THE_DATABASE.lock().await;
    let db = pool().await;
    let now = OffsetDateTime::now_utc();
    let (host, event) = seed_event(&db, now + Duration::hours(3)).await;
    devices::upsert(&db, host, &token(), "sandbox")
        .await
        .unwrap();
    reminders::schedule(&db, event, HOURS_BEFORE, now - Duration::minutes(1))
        .await
        .unwrap();

    drain_once(&db, &Recorder::default(), now).await.unwrap();

    let second = Recorder::default();
    drain_once(&db, &second, now).await.unwrap();

    assert!(
        second.for_event(event).is_empty(),
        "a sent reminder must never go out twice"
    );
}

#[tokio::test]
async fn a_reminder_that_is_not_due_yet_is_left_alone() {
    let _sweeps_run_one_at_a_time = A_SWEEP_CLAIMS_EVERY_DUE_JOB_IN_THE_DATABASE.lock().await;
    let db = pool().await;
    let now = OffsetDateTime::now_utc();
    let (host, event) = seed_event(&db, now + Duration::hours(30)).await;
    devices::upsert(&db, host, &token(), "sandbox")
        .await
        .unwrap();
    reminders::schedule(&db, event, DAY_BEFORE, now + Duration::hours(6))
        .await
        .unwrap();

    let recorder = Recorder::default();
    drain_once(&db, &recorder, now).await.unwrap();

    assert!(
        recorder.for_event(event).is_empty(),
        "a future reminder waits its turn"
    );
    assert_eq!(
        reminders::pending_kinds(&db, event).await.unwrap(),
        vec![DAY_BEFORE.to_owned()]
    );
}

#[tokio::test]
async fn an_event_with_nobody_to_notify_still_settles_the_job() {
    let _sweeps_run_one_at_a_time = A_SWEEP_CLAIMS_EVERY_DUE_JOB_IN_THE_DATABASE.lock().await;
    let db = pool().await;
    let now = OffsetDateTime::now_utc();
    let (_, event) = seed_event(&db, now + Duration::hours(3)).await;
    reminders::schedule(&db, event, HOURS_BEFORE, now - Duration::minutes(1))
        .await
        .unwrap();

    drain_once(&db, &Recorder::default(), now).await.unwrap();

    assert!(
        reminders::pending_kinds(&db, event)
            .await
            .unwrap()
            .is_empty(),
        "a job with nobody to notify must not stay pending forever"
    );
}
