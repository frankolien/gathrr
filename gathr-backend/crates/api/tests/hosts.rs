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

async fn add_host(app: &impl Harness, token: &str, event: Uuid, user_id: Uuid) -> ServiceResponse {
    test::call_service(
        app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event}/hosts"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(json!({ "user_id": user_id }))
            .to_request(),
    )
    .await
}

async fn edit_title(app: &impl Harness, token: &str, event: Uuid, title: &str) -> ServiceResponse {
    test::call_service(
        app,
        test::TestRequest::patch()
            .uri(&format!("/v1/events/{event}"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .set_json(json!({ "title": title }))
            .to_request(),
    )
    .await
}

#[actix_web::test]
async fn creating_an_event_makes_the_creator_its_owner() {
    let state = state().await;
    let app = service!(state);
    let host = sign_in(&app, "Amara Chukwu").await;
    let event = publish_event(&app, &host.token, "Rooftop Supper").await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event}/hosts"))
            .insert_header(("authorization", format!("Bearer {}", host.token)))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);

    let roster = body_json(response).await;
    assert_eq!(roster["hosts"].as_array().unwrap().len(), 1);
    assert_eq!(roster["hosts"][0]["role"], "owner");
    assert_eq!(roster["hosts"][0]["display_name"], "Amara Chukwu");
}

#[actix_web::test]
async fn a_co_host_can_run_the_event_and_a_stranger_cannot() {
    let state = state().await;
    let app = service!(state);
    let owner = sign_in(&app, "Amara Chukwu").await;
    let helper = sign_in(&app, "Tunde Bello").await;
    let stranger = sign_in(&app, "Passing Stranger").await;
    let event = publish_event(&app, &owner.token, "Book Club").await;

    assert_eq!(
        edit_title(&app, &helper.token, event, "Nope")
            .await
            .status(),
        403,
        "before promotion they are just a stranger"
    );

    assert_eq!(
        add_host(&app, &owner.token, event, helper.id)
            .await
            .status(),
        201
    );

    assert_eq!(
        edit_title(&app, &helper.token, event, "Book Club, moved")
            .await
            .status(),
        200,
        "a co-host edits the event like the owner does"
    );
    assert_eq!(
        edit_title(&app, &stranger.token, event, "Nope")
            .await
            .status(),
        403
    );
}

#[actix_web::test]
async fn only_a_host_can_recruit_another_one() {
    let state = state().await;
    let app = service!(state);
    let owner = sign_in(&app, "Amara Chukwu").await;
    let stranger = sign_in(&app, "Passing Stranger").await;
    let victim = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &owner.token, "Listening Party").await;

    assert_eq!(
        add_host(&app, &stranger.token, event, victim.id)
            .await
            .status(),
        403
    );
}

#[actix_web::test]
async fn adding_the_same_co_host_twice_leaves_one_row() {
    let state = state().await;
    let app = service!(state);
    let owner = sign_in(&app, "Amara Chukwu").await;
    let helper = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &owner.token, "Quiet Dinner").await;

    add_host(&app, &owner.token, event, helper.id).await;
    let again = add_host(&app, &owner.token, event, helper.id).await;

    assert_eq!(again.status(), 201);
    assert_eq!(body_json(again).await["hosts"].as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn the_owner_cannot_be_removed_and_a_co_host_can_walk_away() {
    let state = state().await;
    let app = service!(state);
    let owner = sign_in(&app, "Amara Chukwu").await;
    let helper = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &owner.token, "Games Night").await;
    add_host(&app, &owner.token, event, helper.id).await;

    let ousted = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("/v1/events/{event}/hosts/{}", owner.id))
            .insert_header(("authorization", format!("Bearer {}", helper.token)))
            .to_request(),
    )
    .await;
    assert_eq!(
        ousted.status(),
        403,
        "a co-host cannot remove anyone but themselves"
    );

    let self_removal = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("/v1/events/{event}/hosts/{}", helper.id))
            .insert_header(("authorization", format!("Bearer {}", helper.token)))
            .to_request(),
    )
    .await;
    assert_eq!(self_removal.status(), 204);

    assert_eq!(
        edit_title(&app, &helper.token, event, "Nope")
            .await
            .status(),
        403,
        "standing down really removes the power"
    );
}

#[actix_web::test]
async fn an_owner_who_deletes_their_account_hands_the_event_to_a_co_host() {
    let state = state().await;
    let db = state.db.clone();
    let app = service!(state);
    let owner = sign_in(&app, "Amara Chukwu").await;
    let helper = sign_in(&app, "Tunde Bello").await;
    let handed_over = publish_event(&app, &owner.token, "Survives The Owner").await;
    let alone = publish_event(&app, &owner.token, "Nobody Else Runs This").await;
    add_host(&app, &owner.token, handed_over, helper.id).await;

    let erased = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri("/v1/me")
            .insert_header(("authorization", format!("Bearer {}", owner.token)))
            .to_request(),
    )
    .await;
    assert_eq!(erased.status(), 200);
    let outcome = body_json(erased).await;
    assert_eq!(outcome["events_handed_over"], 1);
    assert_eq!(outcome["events_cancelled"], 1);

    let survivor = gathr_infra_db::events::find(&db, handed_over)
        .await
        .unwrap();
    assert_eq!(
        survivor
            .expect("the event must outlive its former owner")
            .host_id,
        helper.id,
        "the co-host inherits the event rather than losing it"
    );
    assert!(
        gathr_infra_db::events::find(&db, alone)
            .await
            .unwrap()
            .is_none(),
        "an event with nobody left to run it goes with the account"
    );

    assert_eq!(
        edit_title(&app, &helper.token, handed_over, "Now mine")
            .await
            .status(),
        200
    );
}

#[actix_web::test]
async fn a_co_hosted_event_shows_up_in_the_hosting_feed() {
    let state = state().await;
    let app = service!(state);
    let owner = sign_in(&app, "Amara Chukwu").await;
    let helper = sign_in(&app, "Tunde Bello").await;
    let event = publish_event(&app, &owner.token, "Shared Load").await;
    add_host(&app, &owner.token, event, helper.id).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/v1/events?filter=hosting")
            .insert_header(("authorization", format!("Bearer {}", helper.token)))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);

    let feed = body_json(response).await;
    assert!(
        feed.as_array()
            .unwrap()
            .iter()
            .any(|event| event["title"] == "Shared Load"),
        "hosting means every event you run, not only the ones you created"
    );
}
