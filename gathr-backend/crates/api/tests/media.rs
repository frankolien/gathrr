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

macro_rules! record_media {
    ($app:expr, $token:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/v1/media")
                .insert_header(("authorization", format!("Bearer {}", $token)))
                .set_json(json!({
                    "public_id": format!("gathr/covers/{}", Uuid::new_v4()),
                    "content_type": "image/jpeg",
                    "width": 1200,
                    "height": 800
                }))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 201);
        Uuid::parse_str(body_json(response).await["id"].as_str().unwrap()).unwrap()
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
                    "title": "Amara's 26th Birthday",
                    "category": "birthday",
                    "starts_at": "2027-09-08T18:00:00Z",
                    "publish_now": true
                }))
                .to_request(),
        )
        .await;
        Uuid::parse_str(body_json(response).await["id"].as_str().unwrap()).unwrap()
    }};
}

#[actix_web::test]
async fn an_upload_ticket_carries_a_signature_but_never_the_api_secret() {
    let state = state().await;
    let secret = state.config.cloudinary_api_secret.clone();
    let app = service!(state);
    let token = sign_in!(app, "Amara Chukwu");

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/media/sign")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "purpose": "cover" }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 200);
    let ticket = body_json(response).await;
    assert_eq!(ticket["folder"], "gathr/covers");
    assert_eq!(ticket["signature"].as_str().unwrap().len(), 40);
    assert!(ticket["upload_url"]
        .as_str()
        .unwrap()
        .starts_with("https://api.cloudinary.com/"));

    let rendered = ticket.to_string();
    assert!(
        secret.is_empty() || !rendered.contains(&secret),
        "the ticket goes to the phone and must never carry the api secret"
    );
}

#[actix_web::test]
async fn an_unsigned_caller_cannot_get_an_upload_ticket() {
    let state = state().await;
    let app = service!(state);

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/media/sign")
            .set_json(json!({ "purpose": "cover" }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 401);
}

#[actix_web::test]
async fn uploads_are_confined_to_the_folders_this_app_owns() {
    let state = state().await;
    let app = service!(state);
    let token = sign_in!(app, "Amara Chukwu");

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/media/sign")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "purpose": "../../etc/passwd" }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 422);
}

#[actix_web::test]
async fn only_an_image_can_be_recorded() {
    let state = state().await;
    let app = service!(state);
    let token = sign_in!(app, "Amara Chukwu");

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/media")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "public_id": "gathr/covers/x", "content_type": "application/pdf" }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 422);
}

