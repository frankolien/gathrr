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
        body_json(response).await["access_token"]
            .as_str()
            .unwrap()
            .to_owned()
    }};
}

macro_rules! register_device {
    ($app:expr, $token:expr, $apns:expr) => {
        test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/v1/devices")
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .set_json(json!({ "apns_token": $apns, "environment": "sandbox" }))
                .to_request(),
        )
        .await
    };
}

fn apns_token() -> String {
    Uuid::new_v4().simple().to_string().repeat(2)
}

#[actix_web::test]
async fn a_device_registers_once_and_re_registering_the_same_token_does_not_duplicate_it() {
    let state = state().await;
    let app = service!(state);
    let token = sign_in!(app, "Amara Chukwu");
    let apns = apns_token();

    let first = register_device!(app, token, &apns);
    assert_eq!(first.status(), 201);
    let id = body_json(first).await["id"].as_str().unwrap().to_owned();

    let again = register_device!(app, token, &apns);
    assert_eq!(again.status(), 201);
    assert_eq!(
        body_json(again).await["id"].as_str().unwrap(),
        id,
        "the same hardware must keep the same device row across launches"
    );
}

#[actix_web::test]
async fn something_that_is_not_a_device_token_is_refused() {
    let state = state().await;
    let app = service!(state);
    let token = sign_in!(app, "Amara Chukwu");

    let response = register_device!(app, token, "not-a-token");
    assert_eq!(response.status(), 422);
    assert_eq!(
        body_json(response).await["error"]["code"],
        "validation_failed"
    );
}

#[actix_web::test]
async fn a_signed_out_caller_cannot_register_a_device() {
    let state = state().await;
    let app = service!(state);

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/devices")
            .set_json(json!({ "apns_token": apns_token() }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 401);
}

