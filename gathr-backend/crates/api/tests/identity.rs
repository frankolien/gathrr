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

#[actix_web::test]
async fn an_unknown_sign_in_provider_is_rejected_before_any_network_call() {
    let state = state().await;
    let app = service!(state);

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/oauth")
            .set_json(json!({ "provider": "facebook", "id_token": "irrelevant" }))
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), 422);
    assert_eq!(
        body_json(response).await["error"]["code"],
        "validation_failed"
    );
}

#[actix_web::test]
async fn a_signed_in_user_can_fill_in_the_profile_guests_will_see() {
    let state = state().await;
    let app = service!(state);

    let signed_in = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/dev")
            .set_json(json!({ "display_name": "Placeholder Name" }))
            .to_request(),
    )
    .await;
    let token = body_json(signed_in).await["access_token"]
        .as_str()
        .expect("dev sign-in must return an access token")
        .to_owned();

    let saved = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri("/v1/me")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({
                "display_name": "  Amara Chukwu  ",
                "bio": "  Lover of themed parties  "
            }))
            .to_request(),
    )
    .await;

    assert_eq!(saved.status(), 200);
    let profile = body_json(saved).await;
    assert_eq!(profile["display_name"], "Amara Chukwu");
    assert_eq!(profile["bio"], "Lover of themed parties");

    let partial = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri("/v1/me")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "display_name": "Amara C." }))
            .to_request(),
    )
    .await;
    assert_eq!(
        body_json(partial).await["bio"],
        "Lover of themed parties",
        "omitting the bio must not wipe it"
    );

    let cleared = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri("/v1/me")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "bio": null }))
            .to_request(),
    )
    .await;
    assert!(body_json(cleared).await["bio"].is_null());

    let blank = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri("/v1/me")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "display_name": "   " }))
            .to_request(),
    )
    .await;

    assert_eq!(blank.status(), 422);
}

macro_rules! request_code {
    ($app:expr, $destination:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/v1/auth/otp/request")
                .set_json(json!({ "channel": "email", "destination": $destination }))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 202);
        body_json(response).await["development_code"]
            .as_str()
            .expect("development builds must reveal the code")
            .to_owned()
    }};
}

#[actix_web::test]
async fn a_code_sent_to_an_email_signs_that_person_in() {
    let state = state().await;
    let app = service!(state);
    let address = format!("amara+{}@example.com", Uuid::new_v4());

    let code = request_code!(app, &address);

    let verified = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/otp/verify")
            .set_json(json!({ "channel": "email", "destination": &address, "code": code }))
            .to_request(),
    )
    .await;

    assert_eq!(verified.status(), 200);
    assert!(body_json(verified).await["access_token"].is_string());
}

#[actix_web::test]
async fn the_same_address_in_a_different_case_reaches_the_same_account() {
    let state = state().await;
    let app = service!(state);
    let address = format!("Amara+{}@Example.com", Uuid::new_v4());

    let first = request_code!(app, &address);
    let signed_in = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/otp/verify")
            .set_json(json!({ "channel": "email", "destination": &address, "code": first }))
            .to_request(),
    )
    .await;
    let original = body_json(signed_in).await["user_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let shouted = address.to_uppercase();
    let second = request_code!(app, &shouted);
    let again = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/otp/verify")
            .set_json(json!({ "channel": "email", "destination": &shouted, "code": second }))
            .to_request(),
    )
    .await;

    assert_eq!(
        body_json(again).await["user_id"].as_str().unwrap(),
        original
    );
}

