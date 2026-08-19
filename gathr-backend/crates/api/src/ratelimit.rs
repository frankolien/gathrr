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
