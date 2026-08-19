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

async fn body_text(response: ServiceResponse) -> String {
    let bytes = to_bytes(response.into_body())
        .await
        .expect("a body must be readable");
    String::from_utf8_lossy(&bytes).into_owned()
}

fn event_body() -> Value {
    json!({
        "title": "Amara's 26th Birthday",
        "category": "birthday",
        "location_name": "Victoria Island, Lagos",
        "starts_at": "2026-09-08T18:00:00Z",
        "publish_now": true
    })
}

#[actix_web::test]
async fn the_demo_spine_works_end_to_end() {
    let state = state().await;
    let app = service!(state);

    let signed_in = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/auth/dev")
            .set_json(json!({ "display_name": "Amara Chukwu" }))
            .to_request(),
    )
    .await;
    assert!(signed_in.status().is_success());
    let tokens = body_json(signed_in).await;
    let bearer = format!("Bearer {}", tokens["access_token"].as_str().unwrap());

    let key = Uuid::new_v4().to_string();
    let created = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/events")
            .insert_header(("authorization", bearer.clone()))
            .insert_header(("idempotency-key", key.clone()))
            .set_json(event_body())
            .to_request(),
    )
    .await;
    assert_eq!(created.status().as_u16(), 201);
    let event = body_json(created).await;
    let event_id = event["id"].as_str().unwrap().to_owned();
    assert_eq!(event["status"], "published");

    let replayed = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/events")
            .insert_header(("authorization", bearer.clone()))
            .insert_header(("idempotency-key", key.clone()))
            .set_json(event_body())
            .to_request(),
    )
    .await;
    assert_eq!(
        body_json(replayed).await["id"].as_str().unwrap(),
        event_id,
        "a replayed key must return the original event, not create a second one"
    );

    let invite = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/v1/events/{event_id}/invites"))
            .insert_header(("authorization", bearer.clone()))
            .set_json(json!({}))
            .to_request(),
    )
    .await;
    assert_eq!(invite.status().as_u16(), 201);
    let invite = body_json(invite).await;
    let code = invite["code"].as_str().unwrap().to_owned();
    assert_eq!(code.len(), 10);

    let page = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/i/{code}"))
            .to_request(),
    )
    .await;
    assert!(page.status().is_success());
    let html = body_text(page).await;
    assert!(html.contains("26th Birthday"));
    assert!(
        html.contains("7:00 PM"),
        "times render in the event timezone"
    );
    assert!(
        html.contains("og:title"),
        "link previews need open graph tags"
    );

    let rsvp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/i/{code}/rsvp"))
            .set_form([
                ("display_name", "Tunde Bello"),
                ("status", "going"),
                ("plus_ones", "1"),
            ])
            .to_request(),
    )
    .await;
    assert!(rsvp.status().is_success());
    assert!(body_text(rsvp).await.contains("going</h1>"));

    let guests = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/events/{event_id}/guests"))
            .insert_header(("authorization", bearer))
            .to_request(),
    )
    .await;
    let guests = body_json(guests).await;
    assert_eq!(guests["going"], 1, "one person is going");
    assert_eq!(guests["seats_taken"], 2, "but they consume two seats");
    assert_eq!(guests["guests"][0]["display_name"], "Tunde Bello");
}

#[actix_web::test]
async fn mutating_requests_require_an_idempotency_key() {
    let state = state().await;
    let app = service!(state);

    let tokens = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/auth/dev")
                .set_json(json!({ "display_name": "Key Tester" }))
                .to_request(),
        )
        .await,
    )
    .await;
    let bearer = format!("Bearer {}", tokens["access_token"].as_str().unwrap());

    let missing = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/events")
            .insert_header(("authorization", bearer.clone()))
            .set_json(event_body())
            .to_request(),
    )
    .await;
    assert_eq!(missing.status().as_u16(), 422);

    let key = Uuid::new_v4().to_string();
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/events")
            .insert_header(("authorization", bearer.clone()))
            .insert_header(("idempotency-key", key.clone()))
            .set_json(event_body())
            .to_request(),
    )
    .await;

    let conflicting = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/v1/events")
            .insert_header(("authorization", bearer))
            .insert_header(("idempotency-key", key))
            .set_json(json!({ "title": "Something else", "starts_at": "2026-09-09T18:00:00Z" }))
            .to_request(),
    )
    .await;
    assert_eq!(
        conflicting.status().as_u16(),
        409,
        "the same key with a different payload must not silently replay"
    );
    assert_eq!(
        body_json(conflicting).await["error"]["code"],
        "idempotency_conflict"
    );
}

#[actix_web::test]
async fn protected_routes_reject_anonymous_and_foreign_callers() {
    let state = state().await;
    let app = service!(state);

    let anonymous =
        test::call_service(&app, test::TestRequest::get().uri("/v1/me").to_request()).await;
    assert_eq!(anonymous.status().as_u16(), 401);

    let garbage = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/v1/me")
            .insert_header(("authorization", "Bearer not-a-token"))
            .to_request(),
    )
    .await;
    assert_eq!(garbage.status().as_u16(), 401);
    assert_eq!(body_json(garbage).await["error"]["code"], "unauthenticated");

    let host = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/auth/dev")
                .set_json(json!({ "display_name": "Owner" }))
                .to_request(),
        )
        .await,
    )
    .await;
    let stranger = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/auth/dev")
                .set_json(json!({ "display_name": "Stranger" }))
                .to_request(),
        )
        .await,
    )
    .await;

    let event = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/events")
                .insert_header((
                    "authorization",
                    format!("Bearer {}", host["access_token"].as_str().unwrap()),
                ))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(event_body())
                .to_request(),
        )
        .await,
    )
    .await;

    let forbidden = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!(
                "/v1/events/{}/cancel",
                event["id"].as_str().unwrap()
            ))
            .insert_header((
                "authorization",
                format!("Bearer {}", stranger["access_token"].as_str().unwrap()),
            ))
            .to_request(),
    )
    .await;
    assert_eq!(forbidden.status().as_u16(), 403);
}

#[actix_web::test]
async fn invite_codes_are_forgiving_to_type_but_not_to_guess() {
    let state = state().await;
    let app = service!(state);

    let tokens = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/auth/dev")
                .set_json(json!({ "display_name": "Code Host" }))
                .to_request(),
        )
        .await,
    )
    .await;
    let bearer = format!("Bearer {}", tokens["access_token"].as_str().unwrap());

    let event = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/v1/events")
                .insert_header(("authorization", bearer.clone()))
                .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
                .set_json(event_body())
                .to_request(),
        )
        .await,
    )
    .await;

    let invite = body_json(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri(&format!(
                    "/v1/events/{}/invites",
                    event["id"].as_str().unwrap()
                ))
                .insert_header(("authorization", bearer))
                .set_json(json!({}))
                .to_request(),
        )
        .await,
    )
    .await;
    let code = invite["code"].as_str().unwrap().to_owned();

    let lowercase = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/v1/invites/{}", code.to_lowercase()))
            .to_request(),
    )
    .await;
    assert!(
        lowercase.status().is_success(),
        "a guest retyping a code in lowercase must still get in"
    );

    for guess in ["ZZZZZZZZZZ", "SHORT", "UUUUUUUUUU"] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/v1/invites/{guess}"))
                .to_request(),
        )
        .await;
        assert_eq!(
            response.status().as_u16(),
            404,
            "invalid and unknown codes must be indistinguishable"
        );
    }
}
