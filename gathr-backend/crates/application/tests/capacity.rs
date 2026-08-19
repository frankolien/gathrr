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

#[tokio::test]
async fn plus_ones_consume_seats_and_the_actor_is_not_double_counted() {
    let db = pool().await;
    let host = user(&db, "Amara Chukwu").await;
    let event_id = event_with_capacity(&db, host, Some(4)).await;

    let first = user(&db, "Tunde Bello").await;
    let view = rsvps::submit(&db, going(event_id, first, 2))
        .await
        .expect("three seats fit inside four");
    assert_eq!(view.status, RsvpStatus::Going);
    assert_eq!(view.seats_remaining, Some(1));

    let second = user(&db, "Chidi Okonkwo").await;
    let error = rsvps::submit(&db, going(event_id, second, 1))
        .await
        .expect_err("two more seats do not fit");
    assert!(matches!(
        error,
        AppError::Domain(DomainError::CapacityExceeded { seats_short: 1 })
    ));

    let reduced = rsvps::submit(&db, going(event_id, first, 0))
        .await
        .expect("shrinking an existing rsvp must not be blocked by the seats it already holds");
    assert_eq!(reduced.status, RsvpStatus::Going);
    assert_eq!(reduced.seats_remaining, Some(3));
}

#[tokio::test]
async fn a_waitlisted_guest_is_promoted_only_when_a_seat_frees() {
    let db = pool().await;
    let host = user(&db, "Amara Chukwu").await;
    let event_id = event_with_capacity(&db, host, Some(1)).await;

    let holder = user(&db, "Ngozi Eze").await;
    rsvps::submit(&db, going(event_id, holder, 0))
        .await
        .expect("the first guest takes the only seat");

    let waiting = user(&db, "Bola Ade").await;
    let queued = rsvps::submit(
        &db,
        SubmitRsvp {
            accept_waitlist: true,
            ..going(event_id, waiting, 0)
        },
    )
    .await
    .expect("opting in must waitlist rather than fail");
    assert_eq!(queued.status, RsvpStatus::Waitlisted);

    let blocked = rsvps::promote(&db, event_id, host, waiting)
        .await
        .expect_err("promotion must fail while the event is still full");
    assert!(matches!(
        blocked,
        AppError::Domain(DomainError::CapacityExceeded { .. })
    ));

    rsvps::submit(
        &db,
        SubmitRsvp {
            status: RsvpStatus::Declined,
            ..going(event_id, holder, 0)
        },
    )
    .await
    .expect("the holder gives up their seat");

    let promoted = rsvps::promote(&db, event_id, host, waiting)
        .await
        .expect("promotion succeeds once a seat frees");
    assert_eq!(promoted.status, RsvpStatus::Going);
}

#[tokio::test]
async fn an_uncapped_event_admits_everyone() {
    let db = pool().await;
    let host = user(&db, "Amara Chukwu").await;
    let event_id = event_with_capacity(&db, host, None).await;

    for index in 0..8 {
        let guest = user(&db, &format!("Open Guest {index}")).await;
        let view = rsvps::submit(&db, going(event_id, guest, 2))
            .await
            .expect("an uncapped event never rejects");
        assert_eq!(view.status, RsvpStatus::Going);
        assert_eq!(view.seats_remaining, None);
    }
}

