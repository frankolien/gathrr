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

