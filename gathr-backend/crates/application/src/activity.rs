use gathr_domain::RsvpStatus;
use gathr_infra_db::notifications::{self as feed_rows, NotificationRecord};
use gathr_infra_db::Db;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;

pub const RSVP_ACCEPTED: &str = "rsvp_accepted";
pub const RSVP_DECLINED: &str = "rsvp_declined";
pub const RSVP_WAITLISTED: &str = "rsvp_waitlisted";
pub const EVENT_PUBLISHED: &str = "event_published";
pub const EVENT_CANCELLED: &str = "event_cancelled";
pub const EVENT_REMINDER: &str = "event_reminder";

pub const DEFAULT_PAGE_SIZE: i64 = 40;
pub const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Clone)]
pub struct Feed {
    pub entries: Vec<NotificationRecord>,
    pub unread: i64,
}

pub fn kind_for(status: RsvpStatus) -> Option<&'static str> {
    match status {
        RsvpStatus::Going => Some(RSVP_ACCEPTED),
        RsvpStatus::Declined => Some(RSVP_DECLINED),
        RsvpStatus::Waitlisted => Some(RSVP_WAITLISTED),
        RsvpStatus::Invited | RsvpStatus::Maybe => None,
    }
}

