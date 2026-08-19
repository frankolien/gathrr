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

async fn event_with_capacity(db: &Db, host: Uuid, capacity: Option<i32>) -> Uuid {
    events::create(
        db,
        CreateEvent {
            host_id: host,
            title: "Group Therapy".to_owned(),
            category: Category::Meetup,
            description: None,
            location_name: Some("Ikeja, Lagos".to_owned()),
            starts_at: OffsetDateTime::now_utc() + Duration::days(9),
            ends_at: None,
            timezone: "Africa/Lagos".to_owned(),
            capacity,
            max_plus_ones: 2,
            publish_now: true,
        },
    )
    .await
    .expect("an event must be created")
    .event
    .id
}

fn going(event_id: Uuid, actor_id: Uuid, plus_ones: i32) -> SubmitRsvp {
    SubmitRsvp {
        event_id,
        actor_id,
        status: RsvpStatus::Going,
        plus_ones,
        accept_waitlist: false,
        invite_id: None,
    }
}

#[tokio::test]
async fn concurrent_rsvps_never_oversell_the_event() {
    let db = Arc::new(pool().await);
    let host = user(&db, "Amara Chukwu").await;
    let capacity = 5;
    let event_id = event_with_capacity(&db, host, Some(capacity)).await;

    let mut guests = Vec::new();
    for index in 0..20 {
        guests.push(user(&db, &format!("Guest {index}")).await);
    }

    let mut handles = Vec::new();
    for guest in guests {
        let db = Arc::clone(&db);
        handles.push(tokio::spawn(async move {
            rsvps::submit(&db, going(event_id, guest, 0)).await
        }));
    }

    let mut admitted = 0;
    let mut rejected = 0;
    for handle in handles {
        match handle.await.expect("the task must not panic") {
            Ok(view) => {
                assert_eq!(view.status, RsvpStatus::Going);
                admitted += 1;
            }
            Err(AppError::Domain(DomainError::CapacityExceeded { .. })) => rejected += 1,
            Err(other) => panic!("unexpected failure: {other}"),
        }
    }

    assert_eq!(admitted, capacity, "exactly the capacity must be admitted");
    assert_eq!(rejected, 20 - capacity);

    let guests = rsvps::guest_list(&db, event_id, host)
        .await
        .expect("the host can read the guest list");
    let seats: i32 = guests
        .iter()
        .filter(|guest| guest.status == RsvpStatus::Going)
        .map(|guest| 1 + guest.plus_ones)
        .sum();
    assert_eq!(seats, capacity, "the stored seat count must match capacity");
}

