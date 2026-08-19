pub mod auth;
pub mod events;
pub mod invites;
pub mod media;
pub mod messages;
pub mod notifications;
pub mod rsvps;
pub mod web;

use actix_web::{web as axw, HttpResponse};

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

