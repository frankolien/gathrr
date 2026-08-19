use actix_web::{web, App};
use awc::ws;
use futures_util::{SinkExt, StreamExt};
use gathr_api::{routes, AppState};
use gathr_common::Config;
use serde_json::{json, Value};
use uuid::Uuid;

async fn shared_state() -> web::Data<AppState> {
    let _ = dotenvy::from_filename("../../.env");
    let config = Config::from_env().expect("the test environment must be configured");
    let db = gathr_infra_db::connect(&config.database_url, 8)
        .await
        .expect("postgres must be reachable");
    web::Data::new(AppState::new(db, config))
}

#[actix_web::test]
async fn a_message_posted_over_http_arrives_on_an_open_socket() {
    let state = shared_state().await;
    let server = {
        let state = state.clone();
        actix_test::start(move || {
            App::new()
                .app_data(state.clone())
                .configure(routes::configure)
        })
    };

    let client = awc::Client::new();
    let token: String = {
        let mut response = client
            .post(server.url("/v1/auth/dev"))
            .send_json(&json!({ "display_name": "Amara Chukwu" }))
            .await
            .expect("dev sign-in must succeed");
        response.json::<Value>().await.unwrap()["access_token"]
            .as_str()
            .unwrap()
            .to_owned()
    };

    let event_id: Uuid = {
        let mut response = client
            .post(server.url("/v1/events"))
            .insert_header(("authorization", format!("Bearer {token}")))
            .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
            .send_json(&json!({
                "title": "Group Therapy",
                "category": "meetup",
                "starts_at": "2027-09-08T18:00:00Z",
                "publish_now": true
            }))
            .await
            .expect("publishing must succeed");
        Uuid::parse_str(
            response.json::<Value>().await.unwrap()["id"]
                .as_str()
                .unwrap(),
        )
        .unwrap()
    };

    let (_, mut socket) = awc::Client::new()
        .ws(server.url(&format!("/v1/events/{event_id}/stream")))
        .header("authorization", format!("Bearer {token}"))
        .connect()
        .await
        .expect("an authorized reader must be able to open the stream");

    client
        .post(server.url(&format!("/v1/events/{event_id}/messages")))
        .insert_header(("authorization", format!("Bearer {token}")))
        .insert_header(("idempotency-key", Uuid::new_v4().to_string()))
        .send_json(&json!({ "body": "Doors at 7, come hungry" }))
        .await
        .expect("posting must succeed");

    let frame = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
        .await
        .expect("the broadcast must arrive before the timeout")
        .expect("the socket must stay open")
        .expect("the frame must be readable");

    let ws::Frame::Text(bytes) = frame else {
        panic!("expected a text frame carrying the message");
    };
    let delivered: Value = serde_json::from_slice(&bytes).expect("the frame must be json");

    assert_eq!(delivered["body"], "Doors at 7, come hungry");
    assert_eq!(delivered["seq"], 1);
    assert_eq!(delivered["sender_display_name"], "Amara Chukwu");

    let _ = socket.send(ws::Message::Close(None)).await;
}

