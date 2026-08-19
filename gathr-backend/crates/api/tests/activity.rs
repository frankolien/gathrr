use actix_web::body::to_bytes;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{test, web, App};
use gathr_api::{routes, AppState};
use gathr_common::Config;
use serde_json::{json, Value};
use uuid::Uuid;

async fn state() -> web::Data<AppState> {
    let _ = dotenvy::from_filename("../../.env");
    let config = Config::from_env().expect("the test environment must be configured");
    let db = gathr_infra_db::connect(&config.database_url, 8)
        .await
        .expect("postgres must be reachable");
    web::Data::new(AppState::new(db, config))
}

macro_rules! service {
    ($state:expr) => {
        test::init_service(
            App::new()
                .app_data($state.clone())
                .configure(routes::configure),
        )
        .await
    };
}

async fn body_json(response: ServiceResponse) -> Value {
    let bytes = to_bytes(response.into_body())
        .await
        .expect("a body must be readable");
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

trait Harness:
    Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>
{
}

impl<T> Harness for T where
    T: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>
{
}

async fn sign_in(app: &impl Harness, name: &str) -> String {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri("/v1/auth/dev")
            .set_json(json!({ "display_name": name }))
            .to_request(),
    )
    .await;
    body_json(response).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn publish_event(app: &impl Harness, token: &str, title: &str) -> Uuid {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri("/v1/events")
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({
                "title": title,
                "category": "party",
                "starts_at": "2027-09-08T18:00:00Z",
                "publish_now": true
            }))
            .to_request(),
    )
    .await;
    Uuid::parse_str(body_json(response).await["id"].as_str().unwrap()).unwrap()
}

async fn rsvp(app: &impl Harness, token: &str, event: Uuid, status: &str) {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/rsvp"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({ "status": status, "plus_ones": 0, "accept_waitlist": false }))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);
}

async fn post_message(app: &impl Harness, token: &str, event: Uuid, body: &str) {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/messages"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({ "body": body }))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 201);
}

async fn feed(app: &impl Harness, token: &str) -> Value {
    let response = test::call_service(
        app,
        test::TestRequest::get()
            .uri("/v1/notifications")
            .insert_header(("authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);
    body_json(response).await
}

fn entries_of_kind<'a>(feed: &'a Value, kind: &str, event: Uuid) -> Vec<&'a Value> {
    feed["notifications"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["kind"] == kind && entry["event_id"] == event.to_string())
        .collect()
}

#[actix_web::test]
async fn saying_yes_puts_the_guest_by_name_in_the_hosts_feed() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let guest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host, "Rooftop Supper").await;

    rsvp(&app, &guest, event, "going").await;

    let feed = feed(&app, &host).await;
    let accepted = entries_of_kind(&feed, "rsvp_accepted", event);
    assert_eq!(accepted.len(), 1, "the host should hear about the yes once");
    assert_eq!(accepted[0]["actor_display_name"], "Tunde Bello");
    assert_eq!(accepted[0]["event_title"], "Rooftop Supper");
    assert_eq!(accepted[0]["read"], false);
}

#[actix_web::test]
async fn publishing_tells_the_host_their_event_is_live() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host, "Listening Party").await;

    let feed = feed(&app, &host).await;
    assert_eq!(entries_of_kind(&feed, "event_published", event).len(), 1);
}

#[actix_web::test]
async fn a_chatty_thread_stays_one_unread_line_until_it_is_read() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let guest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host, "Book Club").await;
    rsvp(&app, &guest, event, "going").await;

    for line in ["are we still on?", "bringing my sister", "see you at 7"] {
        post_message(&app, &guest, event, line).await;
    }

    let before = feed(&app, &host).await;
    assert_eq!(
        entries_of_kind(&before, "message_posted", event).len(),
        1,
        "three messages must not become three rows in the feed"
    );

    let cleared = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/notifications/read")
            .insert_header(("authorization", format!("Bearer {host}")))
            .set_json(json!({ "ids": [] }))
            .to_request(),
    )
    .await;
    assert_eq!(cleared.status(), 200);
    assert_eq!(body_json(cleared).await["unread"], 0);

    post_message(&app, &guest, event, "running ten late").await;
    let after = feed(&app, &host).await;
    assert_eq!(
        entries_of_kind(&after, "message_posted", event).len(),
        2,
        "a message after catching up deserves its own line"
    );
    assert_eq!(after["unread"], 1);
}

#[actix_web::test]
async fn a_guest_never_sees_the_hosts_side_of_the_feed() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let guest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host, "Quiet Dinner").await;
    rsvp(&app, &guest, event, "going").await;

    let feed = feed(&app, &guest).await;
    assert!(
        entries_of_kind(&feed, "rsvp_accepted", event).is_empty(),
        "who said yes is the host's business"
    );
}

#[actix_web::test]
async fn muting_an_event_stops_it_reaching_the_feed() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let guest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host, "Loud Party").await;

    test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/mute"))
            .insert_header(("authorization", format!("Bearer {host}")))
            .set_json(json!({ "muted": true }))
            .to_request(),
    )
    .await;

    rsvp(&app, &guest, event, "going").await;

    let feed = feed(&app, &host).await;
    assert!(entries_of_kind(&feed, "rsvp_accepted", event).is_empty());
}

#[actix_web::test]
async fn cancelling_an_event_reaches_the_guests_and_not_the_host_who_did_it() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let guest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host, "Called Off").await;
    rsvp(&app, &guest, event, "going").await;

    let cancelled = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/cancel"))
            .insert_header(("authorization", format!("Bearer {host}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .to_request(),
    )
    .await;
    assert_eq!(cancelled.status(), 200);

    assert_eq!(
        entries_of_kind(&feed(&app, &guest).await, "event_cancelled", event).len(),
        1
    );
    assert!(
        entries_of_kind(&feed(&app, &host).await, "event_cancelled", event).is_empty(),
        "the host already knows, they pressed the button"
    );
}

#[actix_web::test]
async fn a_signed_out_caller_has_no_feed() {
    let state = state().await;
    let app = service!(state);

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/v1/notifications")
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 401);
}
