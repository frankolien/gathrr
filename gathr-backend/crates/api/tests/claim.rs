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

async fn sign_in(app: &impl Harness, name: &str) -> (String, Uuid) {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri("/v1/auth/dev")
            .set_json(json!({ "display_name": name }))
            .to_request(),
    )
    .await;
    let body = body_json(response).await;
    (
        body["access_token"].as_str().unwrap().to_owned(),
        Uuid::parse_str(body["user_id"].as_str().unwrap()).unwrap(),
    )
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

async fn invite_code(app: &impl Harness, token: &str, event: Uuid) -> String {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/invites"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({}))
            .to_request(),
    )
    .await;
    body_json(response).await["code"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn rsvp_from_the_web(app: &impl Harness, code: &str, name: &str, status: &str) -> String {
    let response = test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/i/{code}/rsvp"))
            .set_form([
                ("display_name", name),
                ("status", status),
                ("plus_ones", "1"),
            ])
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);

    response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "gathr_guest")
        .map(|cookie| cookie.value().to_owned())
        .expect("a first-time web guest must be handed a session token")
}

async fn claim(app: &impl Harness, token: &str, guest_token: &str) -> ServiceResponse {
    test::call_service(
        app,
        test::TestRequest::post()
            .uri("/v1/auth/claim")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "guest_token": guest_token }))
            .to_request(),
    )
    .await
}

async fn guests(app: &impl Harness, token: &str, event: Uuid) -> Value {
    let response = test::call_service(
        app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/guests"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    body_json(response).await
}

#[actix_web::test]
async fn signing_up_after_a_web_rsvp_carries_the_rsvp_across() {
    let state = state().await;
    let db = state.db.clone();
    let app = service!(state);
    let (host, _) = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host, "Rooftop Supper").await;
    let code = invite_code(&app, &host, event).await;

    let guest_token = rsvp_from_the_web(&app, &code, "Tunde Bello", "going").await;
    let (account, account_id) = sign_in(&app, "Tunde Bello").await;

    let claimed = claim(&app, &account, &guest_token).await;
    assert_eq!(claimed.status(), 200);
    let outcome = body_json(claimed).await;
    assert_eq!(outcome["claimed"], true);
    assert_eq!(outcome["rsvps_moved"], 1);

    let roster = guests(&app, &host, event).await;
    let mine = roster["guests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|guest| guest["user_id"] == account_id.to_string())
        .expect("the rsvp must now belong to the signed-up account");
    assert_eq!(mine["status"], "going");
    assert_eq!(mine["plus_ones"], 1);
    assert_eq!(
        roster["guests"].as_array().unwrap().len(),
        1,
        "the shadow guest must not linger beside the account that absorbed it"
    );

    assert_eq!(
        gathr_infra_db::rsvps::find(&db, event, account_id)
            .await
            .unwrap()
            .map(|record| record.status.as_str()),
        Some("going")
    );
}

#[actix_web::test]
async fn a_guest_token_only_works_once() {
    let state = state().await;
    let app = service!(state);
    let (host, _) = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host, "Book Club").await;
    let code = invite_code(&app, &host, event).await;
    let guest_token = rsvp_from_the_web(&app, &code, "Tunde Bello", "going").await;

    let (account, _) = sign_in(&app, "Tunde Bello").await;
    assert_eq!(claim(&app, &account, &guest_token).await.status(), 200);

    let (opportunist, _) = sign_in(&app, "Passing Stranger").await;
    let second = claim(&app, &opportunist, &guest_token).await;
    assert_eq!(second.status(), 404);
    assert_eq!(
        body_json(second).await["error"]["code"],
        "guest_session_invalid"
    );
}

#[actix_web::test]
async fn a_made_up_guest_token_claims_nothing() {
    let state = state().await;
    let app = service!(state);
    let (account, _) = sign_in(&app, "Amara Chukwu").await;

    let response = claim(&app, &account, "not-a-real-session-token").await;
    assert_eq!(response.status(), 404);
    assert_eq!(
        body_json(response).await["error"]["code"],
        "guest_session_invalid"
    );
}

#[actix_web::test]
async fn the_newer_rsvp_wins_when_both_sides_answered() {
    let state = state().await;
    let db = state.db.clone();
    let app = service!(state);
    let (host, _) = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host, "Games Night").await;
    let code = invite_code(&app, &host, event).await;

    let guest_token = rsvp_from_the_web(&app, &code, "Tunde Bello", "declined").await;

    let (account, account_id) = sign_in(&app, "Tunde Bello").await;
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/rsvp"))
            .insert_header(("authorization", format!("Bearer {account}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({ "status": "going", "plus_ones": 0, "accept_waitlist": false }))
            .to_request(),
    )
    .await;

    assert_eq!(claim(&app, &account, &guest_token).await.status(), 200);

    assert_eq!(
        gathr_infra_db::rsvps::find(&db, event, account_id)
            .await
            .unwrap()
            .map(|record| record.status.as_str()),
        Some("going"),
        "the answer given later is the one that survives the merge"
    );
}

#[actix_web::test]
async fn a_guest_cannot_absorb_another_guest() {
    let state = state().await;
    let app = service!(state);
    let (host, _) = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host, "Quiet Dinner").await;
    let code = invite_code(&app, &host, event).await;

    let first = rsvp_from_the_web(&app, &code, "Tunde Bello", "going").await;
    let second = rsvp_from_the_web(&app, &code, "Ada Nwosu", "going").await;

    let shadow_id = gathr_infra_db::tokens::find_guest_session(
        &state.db,
        &gathr_application::auth::hash_token(&second),
    )
    .await
    .unwrap()
    .expect("the second web guest must have a session");

    let borrowed = gathr_application::auth::issue_access(&state.tokens, shadow_id)
        .expect("a token for the shadow user");

    let response = claim(&app, &borrowed, &first).await;
    assert_eq!(response.status(), 403);
}

#[actix_web::test]
async fn claiming_needs_an_account() {
    let state = state().await;
    let app = service!(state);

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/claim")
            .set_json(json!({ "guest_token": "anything" }))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 401);
}
