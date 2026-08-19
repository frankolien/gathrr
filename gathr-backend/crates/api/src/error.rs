use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use gathr_application::AppError;
use gathr_common::ErrorEnvelope;
use std::fmt;
use uuid::Uuid;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        Self(error)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

fn status_for(code: &str) -> StatusCode {
    match code {
        "otp_attempts_exceeded" | "rate_limited" => StatusCode::TOO_MANY_REQUESTS,
        "unauthenticated" | "token_reuse_detected" | "identity_rejected" | "otp_invalid" => {
            StatusCode::UNAUTHORIZED
        }
        "provider_unavailable" | "delivery_failed" => StatusCode::SERVICE_UNAVAILABLE,
        "forbidden" => StatusCode::FORBIDDEN,
        "not_found" | "invite_invalid" | "guest_session_invalid" => StatusCode::NOT_FOUND,
        "invite_expired" => StatusCode::GONE,
        "invite_exhausted"
        | "capacity_exceeded"
        | "event_cancelled"
        | "event_ended"
        | "idempotency_conflict" => StatusCode::CONFLICT,
        "plus_ones_exceeded" | "validation_failed" | "message_invalid" | "report_invalid"
        | "self_target" => StatusCode::UNPROCESSABLE_ENTITY,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        status_for(self.0.code())
    }

    fn error_response(&self) -> HttpResponse {
        let request_id = Uuid::new_v4().to_string();
        let code = self.0.code();

        if self.0.is_internal() {
            tracing::error!(request_id, code, error = %self.0, "request failed");
        } else {
            tracing::info!(request_id, code, error = %self.0, "request rejected");
        }

        let message = if self.0.is_internal() {
            "Something went wrong on our end.".to_owned()
        } else {
            self.0.to_string()
        };

        let mut response = HttpResponse::build(self.status_code()).json(ErrorEnvelope::new(
            code,
            message,
            request_id.clone(),
        ));

        if let Ok(value) = HeaderValue::from_str(&request_id) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-request-id"), value);
        }
        response
    }
}
