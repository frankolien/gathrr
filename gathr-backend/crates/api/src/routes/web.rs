use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header::ContentType;
use actix_web::{web, HttpRequest, HttpResponse};
use gathr_application::rsvps;
use gathr_application::{auth, invites, AppError};
use gathr_domain::{DomainError, RsvpStatus};
use minijinja::context;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extract::{guest_cookie_token, GUEST_COOKIE};
use crate::format::{category_label, category_tint, long_when};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct GuestRsvpForm {
    #[serde(default)]
    pub display_name: String,
    pub status: RsvpStatus,
    #[serde(default)]
    pub plus_ones: Option<i32>,
    #[serde(default)]
    pub accept_waitlist: Option<String>,
}

pub async fn invite_page(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    render_invite(&state, &path.into_inner(), None, "", 0, false).await
}

pub async fn guest_rsvp(
    state: web::Data<AppState>,
    request: HttpRequest,
    path: web::Path<String>,
    form: web::Form<GuestRsvpForm>,
) -> Result<HttpResponse, ApiError> {
    let code = path.into_inner();
    let plus_ones = form.plus_ones.unwrap_or(0);
    let accept_waitlist = form.accept_waitlist.as_deref() == Some("true");

    let existing_guest = match guest_cookie_token(&request) {
        Some(token) => auth::resolve_guest_session(&state.db, &token).await?,
        None => None,
    };

    let outcome = rsvps::submit_as_guest(
        &state.db,
        rsvps::GuestRsvpInput {
            code: &code,
            display_name: &form.display_name,
            existing_guest,
            status: form.status,
            plus_ones,
            accept_waitlist,
            session_ttl_days: state.config.guest_session_ttl_days,
        },
    )
    .await;

    match outcome {
        Ok(result) => {
            let heading = match result.view.status {
                RsvpStatus::Going => "You're going",
                RsvpStatus::Waitlisted => "You're on the waitlist",
                RsvpStatus::Maybe => "Marked as maybe",
                _ => "Thanks for letting the host know",
            };
            let detail = match result.view.status {
                RsvpStatus::Going if plus_ones > 0 => {
                    format!("We've saved {} spots for you.", 1 + plus_ones)
                }
                RsvpStatus::Going => "We've saved your spot.".to_owned(),
                RsvpStatus::Waitlisted => {
                    "The host will let you know if a spot opens up.".to_owned()
                }
                _ => "You can change your answer any time.".to_owned(),
            };

            let html = render(
                &state,
                "message.html",
                context! {
                    heading => heading,
                    detail => detail,
                    back_url => format!("/i/{code}"),
                },
            )?;

            let mut response = HttpResponse::Ok();
            response.content_type(ContentType::html());
            if let Some(token) = result.guest_token {
                response.cookie(
                    Cookie::build(GUEST_COOKIE, token)
                        .path("/i")
                        .http_only(true)
                        .same_site(SameSite::Lax)
                        .max_age(actix_web::cookie::time::Duration::days(
                            state.config.guest_session_ttl_days,
                        ))
                        .finish(),
                );
            }
            Ok(response.body(html))
        }
        Err(AppError::Domain(DomainError::CapacityExceeded { .. })) => {
            render_invite(
                &state,
                &code,
                Some("This event is full. You can join the waitlist and the host will let you know if a spot opens up."),
                &form.display_name,
                plus_ones,
                true,
            )
            .await
        }
        Err(error) => Err(ApiError(error)),
    }
}

async fn render_invite(
    state: &AppState,
    code: &str,
    notice: Option<&str>,
    display_name: &str,
    plus_ones: i32,
    accept_waitlist: bool,
) -> Result<HttpResponse, ApiError> {
    let resolved = invites::resolve(&state.db, code).await?;
    let event = &resolved.event;

    let html = render(
        state,
        "invite.html",
        context! {
            title => event.title.clone(),
            when => long_when(event.starts_at, &event.timezone),
            location => event.location_name.clone(),
            category_label => category_label(event.category),
            tint => category_tint(event.category),
            host_first_name => resolved.host_first_name.clone(),
            going_guests => resolved.going_guests,
            code => resolved.invite.code.clone(),
            invite_url => format!("{}/i/{}", state.config.public_base_url, resolved.invite.code),
            max_plus_ones => event.max_plus_ones,
            notice => notice,
            display_name => display_name,
            plus_ones => plus_ones,
            accept_waitlist => if accept_waitlist { "true" } else { "false" },
        },
    )?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html))
}

fn render(state: &AppState, template: &str, ctx: minijinja::Value) -> Result<String, ApiError> {
    state
        .templates
        .get_template(template)
        .and_then(|template| template.render(ctx))
        .map_err(|error| {
            tracing::error!(%error, template, "template render failed");
            ApiError(AppError::Validation(
                "could not render this page".to_owned(),
            ))
        })
}
