use gathr_domain::{Category, DomainError, EventStatus, RsvpStatus};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: Uuid,
    pub display_name: String,
    pub phone: Option<String>,
    pub is_guest: bool,
    pub bio: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub id: Uuid,
    pub host_id: Uuid,
    pub title: String,
    pub category: Category,
    pub description: Option<String>,
    pub location_name: Option<String>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub timezone: String,
    pub capacity: Option<i32>,
    pub max_plus_ones: i32,
    pub status: EventStatus,
}

#[derive(Debug, Clone)]
pub struct EventSummaryRecord {
    pub event: EventRecord,
    pub going_guests: i32,
    pub preview_guest_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InviteRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub code: String,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct RsvpRecord {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct GuestRecord {
    pub user_id: Uuid,
    pub display_name: String,
    pub status: RsvpStatus,
    pub plus_ones: i32,
}

