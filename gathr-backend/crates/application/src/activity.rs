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

pub async fn page(
    db: &Db,
    user_id: Uuid,
    before: Option<OffsetDateTime>,
    limit: Option<i64>,
) -> Result<Feed, AppError> {
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    Ok(Feed {
        entries: feed_rows::page(db, user_id, before, limit).await?,
        unread: feed_rows::unread_count(db, user_id).await?,
    })
}

pub async fn mark_read(db: &Db, user_id: Uuid, ids: &[Uuid]) -> Result<i64, AppError> {
    feed_rows::mark_read(db, user_id, ids).await?;
    Ok(feed_rows::unread_count(db, user_id).await?)
}

pub async fn record_rsvp(db: &Db, event_id: Uuid, actor_id: Uuid, status: RsvpStatus) {
    let Some(kind) = kind_for(status) else { return };
    settle(
        feed_rows::notify_host(db, event_id, actor_id, kind).await,
        kind,
        event_id,
    );
}

pub async fn record_message(db: &Db, event_id: Uuid, sender_id: Uuid) {
    settle(
        feed_rows::announce_message(db, event_id, sender_id).await,
        "message_posted",
        event_id,
    );
}

pub async fn record_published(db: &Db, event_id: Uuid) {
    settle(
        feed_rows::notify_members(db, event_id, None, EVENT_PUBLISHED).await,
        EVENT_PUBLISHED,
        event_id,
    );
}

