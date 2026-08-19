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

