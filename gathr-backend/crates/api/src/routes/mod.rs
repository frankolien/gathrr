pub mod auth;
pub mod events;
pub mod hosts;
pub mod invites;
pub mod media;
pub mod messages;
pub mod notifications;
pub mod privacy;
pub mod rsvps;
pub mod web;

use actix_web::{web as axw, HttpResponse};

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn ready(state: axw::Data<crate::state::AppState>) -> HttpResponse {
    match sqlx::query_scalar!(r#"SELECT 1 AS "ok!""#)
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "ready" })),
        Err(error) => {
            tracing::error!(%error, "readiness probe could not reach postgres");
            HttpResponse::ServiceUnavailable()
                .json(serde_json::json!({ "status": "database_unreachable" }))
        }
    }
}

pub fn configure(cfg: &mut axw::ServiceConfig) {
    cfg.route("/health", axw::get().to(health))
        .route("/ready", axw::get().to(ready))
        .service(
            axw::scope("/v1")
                .route("/auth/dev", axw::post().to(auth::sign_in_dev))
                .route("/auth/oauth", axw::post().to(auth::sign_in_with_provider))
                .route("/auth/otp/request", axw::post().to(auth::request_code))
                .route("/auth/otp/verify", axw::post().to(auth::verify_code))
                .route("/auth/refresh", axw::post().to(auth::refresh))
                .route("/auth/logout", axw::post().to(auth::logout))
                .route("/me", axw::get().to(auth::me))
                .route("/me", axw::patch().to(auth::update_profile))
                .route("/me", axw::delete().to(privacy::erase))
                .route("/me/export", axw::get().to(privacy::export))
                .route("/reports", axw::post().to(privacy::report))
                .route("/blocks", axw::post().to(privacy::block))
                .route("/blocks", axw::get().to(privacy::blocks))
                .route("/blocks/{uid}", axw::delete().to(privacy::unblock))
                .route("/notifications", axw::get().to(notifications::feed))
                .route(
                    "/notifications/read",
                    axw::post().to(notifications::mark_read),
                )
                .route("/devices", axw::post().to(notifications::register_device))
                .route(
                    "/devices/{id}",
                    axw::delete().to(notifications::forget_device),
                )
                .route("/events/{id}/mute", axw::post().to(notifications::mute))
                .route("/media/sign", axw::post().to(media::sign))
                .route("/media", axw::post().to(media::record))
                .route(
                    "/events/{id}/cover/{media_id}",
                    axw::put().to(media::attach_cover),
                )
                .route("/events", axw::post().to(events::create))
                .route("/events", axw::get().to(events::feed))
                .route("/events/{id}", axw::get().to(events::detail))
                .route("/events/{id}", axw::patch().to(events::edit))
                .route("/events/{id}/publish", axw::post().to(events::publish))
                .route("/events/{id}/cancel", axw::post().to(events::cancel))
                .route("/events/{id}/hosts", axw::get().to(hosts::list))
                .route("/events/{id}/hosts", axw::post().to(hosts::add))
                .route("/events/{id}/hosts/{uid}", axw::delete().to(hosts::remove))
                .route("/events/{id}/invites", axw::post().to(invites::create))
                .route("/events/{id}/invites", axw::get().to(invites::list))
                .route("/events/{id}/messages", axw::post().to(messages::post))
                .route("/events/{id}/messages", axw::get().to(messages::list))
                .route("/events/{id}/stream", axw::get().to(messages::stream))
                .route("/events/{id}/rsvp", axw::post().to(rsvps::submit))
                .route("/events/{id}/guests", axw::get().to(rsvps::guests))
                .route(
                    "/events/{id}/guests/{uid}",
                    axw::delete().to(events::remove_guest),
                )
                .route(
                    "/events/{id}/guests/{uid}/promote",
                    axw::post().to(rsvps::promote),
                )
                .route("/invites/{code}", axw::get().to(invites::resolve)),
        )
        .route("/i/{code}", axw::get().to(web::invite_page))
        .route("/i/{code}/rsvp", axw::post().to(web::guest_rsvp));
}
