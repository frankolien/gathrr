use gathr_domain::{event, Category, EventSchedule, EventStatus};
use gathr_infra_db::events::{self, EventEdit, NewEvent};
use gathr_infra_db::{rsvps, users, Db, DbError, EventRecord, EventSummaryRecord};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct CreateEvent {
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
    pub publish_now: bool,
}

#[derive(Debug, Clone)]
pub struct EventDetail {
    pub event: EventRecord,
    pub observed_status: EventStatus,
    pub going_guests: i32,
    pub preview_guest_names: Vec<String>,
    pub host_display_name: String,
}

pub fn observed_status(event: &EventRecord, now: OffsetDateTime) -> EventStatus {
    EventSchedule {
        starts_at: event.starts_at,
        ends_at: event.ends_at,
    }
    .observed_status(event.status, now)
}

pub async fn create(db: &Db, input: CreateEvent) -> Result<EventSummaryRecord, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("a title is required".to_owned()));
    }
    if let Some(ends_at) = input.ends_at {
        if ends_at <= input.starts_at {
            return Err(AppError::Validation(
                "the end time must be after the start time".to_owned(),
            ));
        }
    }
    if input.capacity.is_some_and(|capacity| capacity <= 0) {
        return Err(AppError::Validation(
            "capacity must be a positive number".to_owned(),
        ));
    }

    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;
    let record = events::insert(
        &mut tx,
        NewEvent {
            host_id: input.host_id,
            title: input.title.trim(),
            category: input.category.as_str(),
            description: input.description.as_deref(),
            location_name: input.location_name.as_deref(),
            starts_at: input.starts_at,
            ends_at: input.ends_at,
            timezone: &input.timezone,
            capacity: input.capacity,
            max_plus_ones: input.max_plus_ones,
        },
    )
    .await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;

    if input.publish_now {
        return publish(db, record.id, input.host_id).await;
    }
    reload(db, record.id).await
}

