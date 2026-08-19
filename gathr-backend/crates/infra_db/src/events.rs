use gathr_domain::{Category, EventStatus};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::{Db, Tx};
use crate::records::{parse_event_status, EventRecord, EventSummaryRecord};

struct EventRow {
    id: Uuid,
    host_id: Uuid,
    title: String,
    category: String,
    description: Option<String>,
    location_name: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: Option<OffsetDateTime>,
    timezone: String,
    capacity: Option<i32>,
    max_plus_ones: i32,
    status: String,
}

impl EventRow {
    fn into_record(self) -> Result<EventRecord, DbError> {
        Ok(EventRecord {
            id: self.id,
            host_id: self.host_id,
            title: self.title,
            category: Category::parse_or_other(&self.category),
            description: self.description,
            location_name: self.location_name,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            timezone: self.timezone,
            capacity: self.capacity,
            max_plus_ones: self.max_plus_ones,
            status: parse_event_status(&self.status)?,
        })
    }
}

struct SummaryRow {
    id: Uuid,
    host_id: Uuid,
    title: String,
    category: String,
    description: Option<String>,
    location_name: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: Option<OffsetDateTime>,
    timezone: String,
    capacity: Option<i32>,
    max_plus_ones: i32,
    status: String,
    going_guests: i32,
    preview_guest_names: Vec<String>,
}

impl SummaryRow {
    fn into_record(self) -> Result<EventSummaryRecord, DbError> {
        Ok(EventSummaryRecord {
            event: EventRecord {
                id: self.id,
                host_id: self.host_id,
                title: self.title,
                category: Category::parse_or_other(&self.category),
                description: self.description,
                location_name: self.location_name,
                starts_at: self.starts_at,
                ends_at: self.ends_at,
                timezone: self.timezone,
                capacity: self.capacity,
                max_plus_ones: self.max_plus_ones,
                status: parse_event_status(&self.status)?,
            },
            going_guests: self.going_guests,
            preview_guest_names: self.preview_guest_names,
        })
    }
}

