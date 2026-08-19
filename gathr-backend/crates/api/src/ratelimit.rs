use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix_web::body::{BoxBody, EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::Next;
use actix_web::{Error, HttpResponse};
use gathr_common::ErrorEnvelope;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Quota {
    pub bucket: &'static str,
    pub allowance: u32,
    pub window: Duration,
}

const VERIFICATION: Quota = Quota {
    bucket: "verification",
    allowance: 8,
    window: Duration::from_secs(900),
};
const AUTHENTICATION: Quota = Quota {
    bucket: "authentication",
    allowance: 30,
    window: Duration::from_secs(60),
};
const INVITE_LOOKUP: Quota = Quota {
    bucket: "invite_lookup",
    allowance: 20,
    window: Duration::from_secs(60),
};
const WRITE: Quota = Quota {
    bucket: "write",
    allowance: 60,
    window: Duration::from_secs(60),
};
const BASELINE: Quota = Quota {
    bucket: "baseline",
    allowance: 300,
    window: Duration::from_secs(60),
};

#[derive(Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn admit(&self, key: &str, quota: Quota) -> Result<(), Duration> {
        let now = Instant::now();
        let mut windows = self.windows.lock().unwrap_or_else(|poisoned| {
            self.windows.clear_poison();
            poisoned.into_inner()
        });

        windows.retain(|_, (started, _)| now.duration_since(*started) < BASELINE.window * 2);

        let entry = windows
            .entry(format!("{}:{key}", quota.bucket))
            .or_insert((now, 0));

        if now.duration_since(entry.0) >= quota.window {
            *entry = (now, 0);
        }

        if entry.1 >= quota.allowance {
            return Err(quota.window - now.duration_since(entry.0));
        }

        entry.1 += 1;
        Ok(())
    }
}

pub fn quota_for(method: &Method, path: &str) -> Quota {
    if path.starts_with("/v1/auth/otp/") {
        VERIFICATION
    } else if path.starts_with("/v1/auth/") {
        AUTHENTICATION
    } else if path.starts_with("/v1/invites/") || path.starts_with("/i/") {
        INVITE_LOOKUP
    } else if method != Method::GET {
        WRITE
    } else {
        BASELINE
    }
}

pub fn caller(request: &ServiceRequest) -> String {
    request
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_owned()
}

pub async fn enforce<B: MessageBody + 'static>(
    request: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<EitherBody<B, BoxBody>>, Error> {
    let quota = quota_for(request.method(), request.path());
    let limiter = request
        .app_data::<actix_web::web::Data<crate::state::AppState>>()
        .map(|state| state.limiter.clone());

    if let Some(limiter) = limiter {
        if let Err(retry_after) = limiter.admit(&caller(&request), quota) {
            return Ok(request
                .into_response(too_many_requests(retry_after))
                .map_into_right_body());
        }
    }

    next.call(request)
        .await
        .map(|response| response.map_into_left_body())
}

fn too_many_requests(retry_after: Duration) -> HttpResponse {
    let request_id = Uuid::new_v4().to_string();
    let seconds = retry_after.as_secs().max(1);

    let mut response = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS).json(ErrorEnvelope::new(
        "rate_limited",
        "You're doing that too often. Try again shortly.".to_owned(),
        request_id,
    ));
    if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("retry-after"), value);
    }
    response
}
