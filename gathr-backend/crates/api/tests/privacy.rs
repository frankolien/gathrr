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

