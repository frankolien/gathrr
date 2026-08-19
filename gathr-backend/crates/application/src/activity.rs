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
