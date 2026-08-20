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

#[actix_web::test]
async fn a_cover_and_its_privacy_choices_survive_the_round_trip() {
    let state = state().await;
    let app = service!(state);
    let token = sign_in!(app, "Amara Chukwu");

    let created = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/events")
                .insert_header(("authorization", format!("Bearer {token}")))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(json!({
                    "title": "Amara's 26th Birthday",
                    "category": "birthday",
                    "starts_at": "2027-09-08T18:00:00Z",
                    "ends_at": "2027-09-08T22:00:00Z",
                    "cover_template_id": "party-confetti",
                    "visibility": "private",
                    "requires_approval": true,
                    "publish_now": true
                }))
                .to_request(),
        )
        .await,
    )
    .await;

    assert_eq!(created["cover_template_id"], "party-confetti");
    assert_eq!(created["visibility"], "private");
    assert_eq!(created["requires_approval"], true);
    assert_eq!(created["ends_at"], "2027-09-08T22:00:00Z");

    let event_id = created["id"].as_str().unwrap().to_owned();
    let detail = body_json(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/v1/events/{event_id}"))
                .insert_header(("authorization", format!("Bearer {token}")))
                .to_request(),
        )
        .await,
    )
    .await;

    assert_eq!(detail["cover_template_id"], "party-confetti");
    assert_eq!(detail["visibility"], "private");
    assert_eq!(detail["requires_approval"], true);

    let edited = body_json(
        test::call_service(
            &app,
            test::TestRequest::patch()
                .uri(&format!("/v1/events/{event_id}"))
                .insert_header(("authorization", format!("Bearer {token}")))
                .set_json(json!({
                    "cover_template_id": "summer-pool",
                    "visibility": "public",
                    "requires_approval": false
                }))
                .to_request(),
        )
        .await,
    )
    .await;

    assert_eq!(edited["cover_template_id"], "summer-pool");
    assert_eq!(edited["visibility"], "public");
    assert_eq!(edited["requires_approval"], false);

    let cleared = body_json(
        test::call_service(
            &app,
            test::TestRequest::patch()
                .uri(&format!("/v1/events/{event_id}"))
                .insert_header(("authorization", format!("Bearer {token}")))
                .set_json(json!({ "cover_template_id": null }))
                .to_request(),
        )
        .await,
    )
    .await;

    assert!(cleared["cover_template_id"].is_null());
    assert_eq!(cleared["visibility"], "public");
}

#[actix_web::test]
async fn an_event_created_without_choices_stays_public_and_open() {
    let state = state().await;
    let app = service!(state);
    let token = sign_in!(app, "Tunde Bakare");

    let created = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/events")
                .insert_header(("authorization", format!("Bearer {token}")))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(json!({
                    "title": "Sunday Hangout",
                    "starts_at": "2027-09-09T18:00:00Z",
                    "publish_now": true
                }))
                .to_request(),
        )
        .await,
    )
    .await;

    assert!(created["cover_template_id"].is_null());
    assert_eq!(created["visibility"], "public");
    assert_eq!(created["requires_approval"], false);
}
