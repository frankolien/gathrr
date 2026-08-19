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

fn collect_summaries(rows: Vec<SummaryRow>) -> Result<Vec<EventSummaryRecord>, DbError> {
    rows.into_iter().map(SummaryRow::into_record).collect()
}

pub struct NewEvent<'a> {
    pub host_id: Uuid,
    pub title: &'a str,
    pub category: &'a str,
    pub description: Option<&'a str>,
    pub location_name: Option<&'a str>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub timezone: &'a str,
    pub capacity: Option<i32>,
    pub max_plus_ones: i32,
}

pub async fn insert(tx: &mut Tx<'_>, new: NewEvent<'_>) -> Result<EventRecord, DbError> {
    let row = sqlx::query_as!(
        EventRow,
        r#"INSERT INTO events
             (host_id, title, category, description, location_name,
              starts_at, ends_at, timezone, capacity, max_plus_ones)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           RETURNING id, host_id, title, category, description, location_name,
                     starts_at, ends_at, timezone, capacity, max_plus_ones,
                     status::text AS "status!""#,
        new.host_id,
        new.title,
        new.category,
        new.description,
        new.location_name,
        new.starts_at,
        new.ends_at,
        new.timezone,
        new.capacity,
        new.max_plus_ones
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    sqlx::query!(
        r#"INSERT INTO event_counters (event_id, last_seq) VALUES ($1, 0)"#,
        row.id
    )
    .execute(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    row.into_record()
}

pub async fn find(db: &Db, id: Uuid) -> Result<Option<EventRecord>, DbError> {
    let row = sqlx::query_as!(
        EventRow,
        r#"SELECT id, host_id, title, category, description, location_name,
                  starts_at, ends_at, timezone, capacity, max_plus_ones,
                  status::text AS "status!"
           FROM events WHERE id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)?;

    row.map(EventRow::into_record).transpose()
}

