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
    let db = gathr_infra_db::connect(&config.database_url, 4)
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
        let body = body_json(response).await;
        (
            body["access_token"].as_str().unwrap().to_owned(),
            body["refresh_token"].as_str().unwrap().to_owned(),
        )
    }};
}

macro_rules! publish_event {
    ($app:expr, $token:expr, $capacity:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/v1/events")
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(json!({
                    "title": "Amara's 26th Birthday",
                    "category": "birthday",
                    "starts_at": "2027-09-08T18:00:00Z",
                    "capacity": $capacity,
                    "publish_now": true
                }))
                .to_request(),
        )
        .await;
        Uuid::parse_str(body_json(response).await["id"].as_str().unwrap()).unwrap()
    }};
}

macro_rules! patch_event {
    ($app:expr, $token:expr, $event:expr, $body:expr) => {
        test::call_service(
            &$app,
            test::TestRequest::patch()
                .uri(&format!("/v1/events/{}", $event))
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .set_json($body)
                .to_request(),
        )
        .await
    };
}

macro_rules! rsvp {
    ($app:expr, $token:expr, $event:expr, $plus_ones:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri(&format!("/v1/events/{}/rsvp", $event))
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(json!({
                    "status": "going",
                    "plus_ones": $plus_ones,
                    "accept_waitlist": false
                }))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 200);
    }};
}

#[actix_web::test]
async fn a_host_can_edit_the_fields_a_guest_reads_and_leave_the_rest_alone() {
    let state = state().await;
    let app = service!(state);
    let (host, _) = sign_in!(app, "Amara Chukwu");
    let event = publish_event!(app, host, 40);

    let response = patch_event!(
        app,
        host,
        event,
        json!({ "title": "Amara's 27th Birthday", "location_name": "Lekki, Lagos" })
    );

    assert_eq!(response.status(), 200);
    let updated = body_json(response).await;
    assert_eq!(updated["title"], "Amara's 27th Birthday");
    assert_eq!(updated["location_name"], "Lekki, Lagos");
    assert_eq!(
        updated["capacity"], 40,
        "a field the caller left out must keep its value"
    );
}

