use time::OffsetDateTime;
use uuid::Uuid;

use gathr_domain::message::{may_post, sanitize};
use gathr_domain::PostingRight;
use gathr_infra_db::{messages, rsvps, Db};

use crate::error::AppError;
use crate::events;

pub const DEFAULT_PAGE_SIZE: i64 = 50;
pub const MAX_PAGE_SIZE: i64 = 200;

#[derive(Debug, Clone)]
pub struct MessageView {
    pub id: Uuid,
    pub event_id: Uuid,
    pub sender_id: Uuid,
    pub sender_display_name: String,
    pub seq: i64,
    pub body: String,
    pub created_at: OffsetDateTime,
}

