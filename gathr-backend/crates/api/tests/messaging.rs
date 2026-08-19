use actix_web::body::to_bytes;
use actix_web::dev::ServiceResponse;
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

macro_rules! sign_in {
    ($app:expr, $name:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/v1/auth/dev")
                .set_json(json!({ "display_name": $name }))
                .to_request(),
        )
        .await;
        body_json(response).await["access_token"].as_str().unwrap().to_owned()
    }};
}

macro_rules! publish_event {
    ($app:expr, $token:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/v1/events")
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(json!({
                    "title": "Group Therapy",
                    "category": "meetup",
                    "starts_at": "2027-09-08T18:00:00Z",
                    "publish_now": true
                }))
                .to_request(),
        )
        .await;
        Uuid::parse_str(body_json(response).await["id"].as_str().unwrap()).unwrap()
    }};
}

macro_rules! post_message {
    ($app:expr, $token:expr, $event:expr, $body:expr) => {{
        test::call_service(
            &$app,
            test::TestRequest::post()
                .uri(&format!("/v1/events/{}/messages", $event))
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(json!({ "body": $body }))
                .to_request(),
        )
        .await
    }};
}

async fn rsvp(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
    token: &str,
    event: Uuid,
) {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/rsvp"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({ "status": "going", "plus_ones": 0, "accept_waitlist": false }))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);
}

#[actix_web::test]
async fn the_first_message_in_an_event_is_sequence_one() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in!(app, "Amara Chukwu");
    let event = publish_event!(app, host);

    let first = post_message!(app, host, event, "Doors at 7, come hungry");
    assert_eq!(first.status(), 201);
    assert_eq!(body_json(first).await["seq"], 1);

    let second = post_message!(app, host, event, "Bring a friend");
    assert_eq!(body_json(second).await["seq"], 2);
}

#[actix_web::test]
async fn a_guest_who_rsvped_can_read_and_reply_but_a_stranger_can_do_neither() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in!(app, "Amara Chukwu");
    let guest = sign_in!(app, "Tunde Bello");
    let stranger = sign_in!(app, "Passing Stranger");
    let event = publish_event!(app, host);

    post_message!(app, host, event, "Doors at 7");
    rsvp(&app, &guest, event).await;

    let replied = post_message!(app, guest, event, "On my way");
    assert_eq!(replied.status(), 201);

    let read = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/messages"))
            .insert_header(("authorization", format!("Bearer {guest}")))
            .to_request(),
    )
    .await;
    assert_eq!(read.status(), 200);
    let payload = body_json(read).await;
    assert_eq!(payload["messages"].as_array().unwrap().len(), 2);
    assert_eq!(payload["latest_seq"], 2);

    let peeked = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/messages"))
            .insert_header(("authorization", format!("Bearer {stranger}")))
            .to_request(),
    )
    .await;
    assert_eq!(
        peeked.status(),
        403,
        "an uninvited reader must not see the thread"
    );

    let intruded = post_message!(app, stranger, event, "hello?");
    assert_eq!(intruded.status(), 403);
}

#[actix_web::test]
async fn paging_after_a_sequence_returns_only_what_the_reader_has_not_seen() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in!(app, "Amara Chukwu");
    let event = publish_event!(app, host);

    for line in ["one", "two", "three"] {
        post_message!(app, host, event, line);
    }

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/messages?after_seq=1"))
            .insert_header(("authorization", format!("Bearer {host}")))
            .to_request(),
    )
    .await;

    let payload = body_json(response).await;
    let bodies: Vec<&str> = payload["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["body"].as_str().unwrap())
        .collect();
    assert_eq!(bodies, vec!["two", "three"]);
}

#[actix_web::test]
async fn an_empty_message_is_refused_and_never_consumes_a_sequence_number() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in!(app, "Amara Chukwu");
    let event = publish_event!(app, host);

    let blank = post_message!(app, host, event, "   \n  ");
    assert_eq!(blank.status(), 422);
    assert_eq!(body_json(blank).await["error"]["code"], "message_invalid");

    let real = post_message!(app, host, event, "Doors at 7");
    assert_eq!(
        body_json(real).await["seq"],
        1,
        "a rejected message must not burn a sequence number"
    );
}

#[actix_web::test]
async fn replaying_a_send_with_the_same_key_does_not_double_post() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in!(app, "Amara Chukwu");
    let event = publish_event!(app, host);
    let key = Uuid::new_v4().to_string();

    let mut seqs = Vec::new();
    for _ in 0..2 {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(&format!("/v1/events/{event}/messages"))
                .insert_header(("authorization", format!("Bearer {host}")))
                .insert_header(("idempotency-key", key.clone()))
                .set_json(json!({ "body": "Doors at 7" }))
                .to_request(),
        )
        .await;
        seqs.push(body_json(response).await["seq"].as_i64().unwrap());
    }

    assert_eq!(seqs[0], seqs[1], "a retry must return the original message");

    let read = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/messages"))
            .insert_header(("authorization", format!("Bearer {host}")))
            .to_request(),
    )
    .await;
    assert_eq!(
        body_json(read).await["messages"].as_array().unwrap().len(),
        1
    );
}
