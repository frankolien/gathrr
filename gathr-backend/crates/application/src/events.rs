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

pub async fn publish(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
) -> Result<EventSummaryRecord, AppError> {
    let record = load_manageable(db, event_id, actor_id).await?;
    let schedule = EventSchedule {
        starts_at: record.starts_at,
        ends_at: record.ends_at,
    };
    let next = event::publish(record.status, &record.title, Some(schedule))?;
    events::set_status(db, event_id, next).await?;
    crate::notifications::plan_reminders(db, event_id).await?;
    crate::activity::record_published(db, event_id).await;
    reload(db, event_id).await
}

pub async fn cancel(
    db: &Db,
    event_id: Uuid,
    actor_id: Uuid,
) -> Result<EventSummaryRecord, AppError> {
    let record = load_manageable(db, event_id, actor_id).await?;
    let next = event::cancel(record.status)?;
    events::set_status(db, event_id, next).await?;
    crate::notifications::cancel_reminders(db, event_id).await?;
    crate::activity::record_cancelled(db, event_id, actor_id).await;
    reload(db, event_id).await
}

pub async fn detail(db: &Db, event_id: Uuid) -> Result<EventDetail, AppError> {
    let summary = events::find_summary(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let host = users::find(db, summary.event.host_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(EventDetail {
        observed_status: observed_status(&summary.event, OffsetDateTime::now_utc()),
        event: summary.event,
        going_guests: summary.going_guests,
        preview_guest_names: summary.preview_guest_names,
        host_display_name: host.display_name,
    })
}

pub async fn feed(
    db: &Db,
    user_id: Uuid,
    filter: &str,
) -> Result<Vec<EventSummaryRecord>, AppError> {
    let now = OffsetDateTime::now_utc();
    match filter {
        "this_week" => Ok(events::feed_this_week(db, user_id, now, now + Duration::days(7)).await?),
        "hosting" => Ok(events::feed_hosting(db, user_id).await?),
        "attending" => Ok(events::feed_attending(db, user_id).await?),
        other => Err(AppError::Validation(format!(
            "unknown feed filter {other}, expected this_week, hosting or attending"
        ))),
    }
}

pub async fn can_manage(db: &Db, event_id: Uuid, actor_id: Uuid) -> Result<bool, AppError> {
    let event = events::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(event.host_id == actor_id)
}

async fn load_manageable(db: &Db, event_id: Uuid, actor_id: Uuid) -> Result<EventRecord, AppError> {
    let record = events::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if record.host_id != actor_id {
        return Err(AppError::Forbidden);
    }
    Ok(record)
}

async fn reload(db: &Db, event_id: Uuid) -> Result<EventSummaryRecord, AppError> {
    events::find_summary(db, event_id)
        .await?
        .ok_or(AppError::NotFound)
}

#[derive(Debug, Clone, Default)]
pub struct EditEvent {
    pub title: Option<String>,
    pub category: Option<Category>,
    pub description: Option<Option<String>>,
    pub location_name: Option<Option<String>>,
    pub starts_at: Option<OffsetDateTime>,
    pub ends_at: Option<Option<OffsetDateTime>>,
    pub timezone: Option<String>,
    pub capacity: Option<Option<i32>>,
    pub max_plus_ones: Option<i32>,
}

