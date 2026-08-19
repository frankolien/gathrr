use gathr_domain::{event, rsvp, CapacityContext, RsvpRequest, RsvpStatus};
use gathr_infra_db::{events, invites, rsvps, tokens, users, Db, DbError, GuestRecord, Tx};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::auth::{hash_token, random_token};
use crate::error::AppError;
use crate::events::observed_status;
use crate::invites::terms;

#[derive(Debug, Clone)]
pub struct SubmitRsvp {
    pub event_id: Uuid,
    pub actor_id: Uuid,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub accept_waitlist: bool,
    pub invite_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct RsvpView {
    pub event_id: Uuid,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub entered_waitlist: bool,
    pub seats_remaining: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct GuestRsvp {
    pub view: RsvpView,
    pub guest_user_id: Uuid,
    pub guest_token: Option<String>,
}

