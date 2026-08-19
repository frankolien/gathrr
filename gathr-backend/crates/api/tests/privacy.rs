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

struct Person {
    token: String,
    id: Uuid,
}

async fn sign_in(app: &impl Harness, name: &str) -> Person {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri("/v1/auth/dev")
            .set_json(json!({ "display_name": name }))
            .to_request(),
    )
    .await;
    let body = body_json(response).await;
    Person {
        token: body["access_token"].as_str().unwrap().to_owned(),
        id: Uuid::parse_str(body["user_id"].as_str().unwrap()).unwrap(),
    }
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

async fn rsvp(app: &impl Harness, token: &str, event: Uuid) {
    test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/rsvp"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({ "status": "going", "plus_ones": 0, "accept_waitlist": false }))
            .to_request(),
    )
    .await;
}

async fn post_message(app: &impl Harness, token: &str, event: Uuid, body: &str) -> Uuid {
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
    Uuid::parse_str(body_json(response).await["id"].as_str().unwrap()).unwrap()
}

async fn thread(app: &impl Harness, token: &str, event: Uuid) -> Value {
    let response = test::call_service(
        app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/messages"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);
    body_json(response).await
}

#[actix_web::test]
async fn a_reported_message_is_recorded_once_per_reporter() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let guest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host.token, "Rooftop Supper").await;
    rsvp(&app, &guest.token, event).await;
    let message = post_message(&app, &guest.token, event, "buy my crypto course").await;

    let mut ids = Vec::new();
    for _ in 0..2 {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/reports")
                .insert_header(("authorization", format!("Bearer {}", host.token)))
                .set_json(json!({
                    "subject": "message",
                    "subject_id": message,
                    "reason": "spam",
                    "detail": "unsolicited advertising in the event thread"
                }))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 201);
        ids.push(body_json(response).await["id"].as_str().unwrap().to_owned());
    }

    assert_eq!(ids[0], ids[1], "reporting twice must not pile up rows");
}

#[actix_web::test]
async fn a_reason_nobody_can_act_on_is_refused() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let stranger = sign_in(&app, "Tunde Bello").await;

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/reports")
            .insert_header(("authorization", format!("Bearer {}", host.token)))
            .set_json(json!({
                "subject": "user",
                "subject_id": stranger.id,
                "reason": "i just do not like them"
            }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 422);
    assert_eq!(body_json(response).await["error"]["code"], "report_invalid");
}

#[actix_web::test]
async fn nobody_reports_or_blocks_themselves() {
    let state = state().await;
    let app = service!(state);
    let person = sign_in(&app, "Amara Chukwu").await;

    for (uri, payload) in [
        (
            "/v1/reports",
            json!({ "subject": "user", "subject_id": person.id, "reason": "spam" }),
        ),
        ("/v1/blocks", json!({ "user_id": person.id })),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(uri)
                .insert_header(("authorization", format!("Bearer {}", person.token)))
                .set_json(payload)
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 422);
        assert_eq!(body_json(response).await["error"]["code"], "self_target");
    }
}

#[actix_web::test]
async fn blocking_someone_hides_their_messages_both_ways() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let pest = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &host.token, "Book Club").await;
    rsvp(&app, &pest.token, event).await;

    post_message(&app, &host.token, event, "doors at seven").await;
    post_message(&app, &pest.token, event, "you are all terrible").await;

    assert_eq!(
        thread(&app, &host.token, event).await["messages"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let blocked = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/blocks")
            .insert_header(("authorization", format!("Bearer {}", host.token)))
            .set_json(json!({ "user_id": pest.id }))
            .to_request(),
    )
    .await;
    assert_eq!(blocked.status(), 201);

    let after = thread(&app, &host.token, event).await;
    assert_eq!(after["messages"].as_array().unwrap().len(), 1);
    assert_eq!(after["messages"][0]["body"], "doors at seven");

    let from_the_other_side = thread(&app, &pest.token, event).await;
    assert_eq!(
        from_the_other_side["messages"].as_array().unwrap().len(),
        1,
        "a block hides the blocker too, not just the blocked"
    );

    let unblocked = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("/v1/blocks/{}", pest.id))
            .insert_header(("authorization", format!("Bearer {}", host.token)))
            .to_request(),
    )
    .await;
    assert_eq!(unblocked.status(), 204);
    assert_eq!(
        thread(&app, &host.token, event).await["messages"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[actix_web::test]
async fn an_export_carries_everything_the_account_holds() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host.token, "Listening Party").await;
    post_message(&app, &host.token, event, "records start at eight").await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/v1/me/export")
            .insert_header(("authorization", format!("Bearer {}", host.token)))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);
    assert!(response
        .headers()
        .get("content-disposition")
        .is_some_and(|value| value.to_str().unwrap().contains("gathr-export.json")));

    let export = body_json(response).await;
    assert_eq!(export["account"]["display_name"], "Amara Chukwu");
    assert_eq!(export["hosted_events"][0]["title"], "Listening Party");
    assert_eq!(export["messages"][0]["body"], "records start at eight");
    assert!(export["exported_at"].as_str().is_some());
}

#[actix_web::test]
async fn deleting_an_account_tombstones_its_messages_and_takes_its_events() {
    let state = state().await;
    let db = state.db.clone();
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let leaver = sign_in(&app, "Tunde Bello").await;

    let their_own = publish_event(&app, &leaver.token, "Their Own Party").await;
    let someone_elses = publish_event(&app, &host.token, "Someone Else's Party").await;
    rsvp(&app, &leaver.token, someone_elses).await;
    post_message(&app, &leaver.token, someone_elses, "cannot wait for this").await;

    let erased = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri("/v1/me")
            .insert_header(("authorization", format!("Bearer {}", leaver.token)))
            .to_request(),
    )
    .await;
    assert_eq!(erased.status(), 200);
    let outcome = body_json(erased).await;
    assert_eq!(outcome["deleted"], true);
    assert_eq!(outcome["events_cancelled"], 1);
    assert_eq!(outcome["messages_redacted"], 1);

    assert!(
        gathr_infra_db::users::find(&db, leaver.id)
            .await
            .unwrap()
            .is_none(),
        "the user row must be gone, not merely flagged"
    );
    assert!(
        gathr_infra_db::events::find(&db, their_own)
            .await
            .unwrap()
            .is_none(),
        "an event nobody hosts any more must not linger"
    );

    let surviving = thread(&app, &host.token, someone_elses).await;
    let tombstone = &surviving["messages"][0];
    assert_eq!(
        tombstone["seq"], 1,
        "the sequence must survive the deletion"
    );
    assert_eq!(tombstone["body"], "");
    assert_eq!(tombstone["redacted"], true);
    assert!(tombstone["sender_display_name"].is_null());
}

#[actix_web::test]
async fn a_signed_out_caller_cannot_reach_the_privacy_surface() {
    let state = state().await;
    let app = service!(state);

    for request in [
        test::TestRequest::get().uri("/v1/me/export").to_request(),
        test::TestRequest::delete().uri("/v1/me").to_request(),
        test::TestRequest::get().uri("/v1/blocks").to_request(),
    ] {
        assert_eq!(test::call_service(&app, request).await.status(), 401);
    }
}
