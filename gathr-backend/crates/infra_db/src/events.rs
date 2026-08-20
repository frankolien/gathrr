use gathr_domain::{Category, EventStatus, EventVisibility};
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
    cover_template_id: Option<String>,
    visibility: String,
    requires_approval: bool,
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
            cover_template_id: self.cover_template_id,
            visibility: EventVisibility::parse_or_public(&self.visibility),
            requires_approval: self.requires_approval,
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
    cover_template_id: Option<String>,
    visibility: String,
    requires_approval: bool,
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
                cover_template_id: self.cover_template_id,
                visibility: EventVisibility::parse_or_public(&self.visibility),
                requires_approval: self.requires_approval,
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
    pub cover_template_id: Option<&'a str>,
    pub visibility: &'a str,
    pub requires_approval: bool,
}

pub async fn insert(tx: &mut Tx<'_>, new: NewEvent<'_>) -> Result<EventRecord, DbError> {
    let row = sqlx::query_as!(
        EventRow,
        r#"INSERT INTO events
             (host_id, title, category, description, location_name,
              starts_at, ends_at, timezone, capacity, max_plus_ones,
              cover_template_id, visibility, requires_approval)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                   $11, ($12::text)::event_visibility, $13)
           RETURNING id, host_id, title, category, description, location_name,
                     starts_at, ends_at, timezone, capacity, max_plus_ones,
                     status::text AS "status!",
                     cover_template_id, visibility::text AS "visibility!", requires_approval"#,
        new.host_id,
        new.title,
        new.category,
        new.description,
        new.location_name,
        new.starts_at,
        new.ends_at,
        new.timezone,
        new.capacity,
        new.max_plus_ones,
        new.cover_template_id,
        new.visibility,
        new.requires_approval
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

    crate::hosts::install_owner(tx, row.id, new.host_id).await?;

    row.into_record()
}

pub async fn find(db: &Db, id: Uuid) -> Result<Option<EventRecord>, DbError> {
    let row = sqlx::query_as!(
        EventRow,
        r#"SELECT id, host_id, title, category, description, location_name,
                  starts_at, ends_at, timezone, capacity, max_plus_ones,
                  status::text AS "status!",
                  cover_template_id, visibility::text AS "visibility!", requires_approval
           FROM events WHERE id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)?;

    row.map(EventRow::into_record).transpose()
}

pub async fn lock(tx: &mut Tx<'_>, id: Uuid) -> Result<Option<EventRecord>, DbError> {
    let row = sqlx::query_as!(
        EventRow,
        r#"SELECT id, host_id, title, category, description, location_name,
                  starts_at, ends_at, timezone, capacity, max_plus_ones,
                  status::text AS "status!",
                  cover_template_id, visibility::text AS "visibility!", requires_approval
           FROM events WHERE id = $1 FOR UPDATE"#,
        id
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    row.map(EventRow::into_record).transpose()
}

pub async fn set_status(db: &Db, id: Uuid, status: EventStatus) -> Result<(), DbError> {
    sqlx::query!(
        r#"UPDATE events SET status = ($2::text)::event_status, updated_at = now()
           WHERE id = $1"#,
        id,
        status.as_str()
    )
    .execute(db)
    .await
    .map_err(DbError::from_sqlx)?;
    Ok(())
}

pub async fn find_summary(db: &Db, id: Uuid) -> Result<Option<EventSummaryRecord>, DbError> {
    let row = sqlx::query_as!(
        SummaryRow,
        r#"SELECT e.id, e.host_id, e.title, e.category, e.description, e.location_name,
                  e.starts_at, e.ends_at, e.timezone, e.capacity, e.max_plus_ones,
                  e.status::text AS "status!",
                  e.cover_template_id, e.visibility::text AS "visibility!", e.requires_approval,
                  COALESCE(g.going_guests, 0) AS "going_guests!",
                  COALESCE(g.preview_names, ARRAY[]::text[]) AS "preview_guest_names!"
           FROM events e
           LEFT JOIN LATERAL (
             SELECT COUNT(*)::int AS going_guests,
                    (ARRAY_AGG(u.display_name ORDER BY r2.updated_at))[1:4] AS preview_names
             FROM rsvps r2
             JOIN users u ON u.id = r2.user_id
             WHERE r2.event_id = e.id AND r2.status = 'going'
           ) g ON TRUE
           WHERE e.id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)?;

    row.map(SummaryRow::into_record).transpose()
}

pub async fn feed_this_week(
    db: &Db,
    user_id: Uuid,
    from: OffsetDateTime,
    until: OffsetDateTime,
) -> Result<Vec<EventSummaryRecord>, DbError> {
    let rows = sqlx::query_as!(
        SummaryRow,
        r#"SELECT e.id, e.host_id, e.title, e.category, e.description, e.location_name,
                  e.starts_at, e.ends_at, e.timezone, e.capacity, e.max_plus_ones,
                  e.status::text AS "status!",
                  e.cover_template_id, e.visibility::text AS "visibility!", e.requires_approval,
                  COALESCE(g.going_guests, 0) AS "going_guests!",
                  COALESCE(g.preview_names, ARRAY[]::text[]) AS "preview_guest_names!"
           FROM events e
           LEFT JOIN rsvps r ON r.event_id = e.id AND r.user_id = $1
           LEFT JOIN LATERAL (
             SELECT COUNT(*)::int AS going_guests,
                    (ARRAY_AGG(u.display_name ORDER BY r2.updated_at))[1:4] AS preview_names
             FROM rsvps r2
             JOIN users u ON u.id = r2.user_id
             WHERE r2.event_id = e.id AND r2.status = 'going'
           ) g ON TRUE
           WHERE (e.host_id = $1 OR r.user_id IS NOT NULL)
             AND e.status IN ('published', 'ongoing')
             AND e.starts_at >= $2 AND e.starts_at < $3
           ORDER BY EXTRACT(EPOCH FROM e.starts_at)
                    - CASE WHEN e.host_id = $1 THEN 3600 ELSE 0 END
           LIMIT 20"#,
        user_id,
        from,
        until
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)?;

    collect_summaries(rows)
}

pub async fn feed_hosting(db: &Db, user_id: Uuid) -> Result<Vec<EventSummaryRecord>, DbError> {
    let rows = sqlx::query_as!(
        SummaryRow,
        r#"SELECT e.id, e.host_id, e.title, e.category, e.description, e.location_name,
                  e.starts_at, e.ends_at, e.timezone, e.capacity, e.max_plus_ones,
                  e.status::text AS "status!",
                  e.cover_template_id, e.visibility::text AS "visibility!", e.requires_approval,
                  COALESCE(g.going_guests, 0) AS "going_guests!",
                  COALESCE(g.preview_names, ARRAY[]::text[]) AS "preview_guest_names!"
           FROM events e
           LEFT JOIN LATERAL (
             SELECT COUNT(*)::int AS going_guests,
                    (ARRAY_AGG(u.display_name ORDER BY r2.updated_at))[1:4] AS preview_names
             FROM rsvps r2
             JOIN users u ON u.id = r2.user_id
             WHERE r2.event_id = e.id AND r2.status = 'going'
           ) g ON TRUE
           WHERE e.status <> 'cancelled'
             AND EXISTS (
               SELECT 1 FROM event_hosts h
               WHERE h.event_id = e.id AND h.user_id = $1
             )
           ORDER BY e.starts_at
           LIMIT 50"#,
        user_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)?;

    collect_summaries(rows)
}

pub async fn feed_attending(db: &Db, user_id: Uuid) -> Result<Vec<EventSummaryRecord>, DbError> {
    let rows = sqlx::query_as!(
        SummaryRow,
        r#"SELECT e.id, e.host_id, e.title, e.category, e.description, e.location_name,
                  e.starts_at, e.ends_at, e.timezone, e.capacity, e.max_plus_ones,
                  e.status::text AS "status!",
                  e.cover_template_id, e.visibility::text AS "visibility!", e.requires_approval,
                  COALESCE(g.going_guests, 0) AS "going_guests!",
                  COALESCE(g.preview_names, ARRAY[]::text[]) AS "preview_guest_names!"
           FROM events e
           JOIN rsvps r ON r.event_id = e.id AND r.user_id = $1
           LEFT JOIN LATERAL (
             SELECT COUNT(*)::int AS going_guests,
                    (ARRAY_AGG(u.display_name ORDER BY r2.updated_at))[1:4] AS preview_names
             FROM rsvps r2
             JOIN users u ON u.id = r2.user_id
             WHERE r2.event_id = e.id AND r2.status = 'going'
           ) g ON TRUE
           WHERE r.status IN ('going', 'maybe', 'waitlisted')
             AND e.status IN ('published', 'ongoing')
           ORDER BY e.starts_at
           LIMIT 50"#,
        user_id
    )
    .fetch_all(db)
    .await
    .map_err(DbError::from_sqlx)?;

    collect_summaries(rows)
}

pub struct EventEdit<'a> {
    pub title: Option<&'a str>,
    pub category: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub location_name: Option<Option<&'a str>>,
    pub starts_at: Option<OffsetDateTime>,
    pub ends_at: Option<Option<OffsetDateTime>>,
    pub timezone: Option<&'a str>,
    pub capacity: Option<Option<i32>>,
    pub max_plus_ones: Option<i32>,
    pub cover_template_id: Option<Option<&'a str>>,
    pub visibility: Option<&'a str>,
    pub requires_approval: Option<bool>,
}

pub async fn update(
    tx: &mut Tx<'_>,
    id: Uuid,
    edit: EventEdit<'_>,
) -> Result<EventRecord, DbError> {
    let row = sqlx::query_as!(
        EventRow,
        r#"UPDATE events SET
             title          = COALESCE($2, title),
             category       = COALESCE($3, category),
             description    = CASE WHEN $4 THEN $5 ELSE description END,
             location_name  = CASE WHEN $6 THEN $7 ELSE location_name END,
             starts_at      = COALESCE($8, starts_at),
             ends_at        = CASE WHEN $9 THEN $10 ELSE ends_at END,
             timezone       = COALESCE($11, timezone),
             capacity       = CASE WHEN $12 THEN $13 ELSE capacity END,
             max_plus_ones  = COALESCE($14, max_plus_ones),
             cover_template_id = CASE WHEN $15 THEN $16 ELSE cover_template_id END,
             visibility     = COALESCE(($17::text)::event_visibility, visibility),
             requires_approval = COALESCE($18, requires_approval)
           WHERE id = $1
           RETURNING id, host_id, title, category, description, location_name,
                     starts_at, ends_at, timezone, capacity, max_plus_ones,
                     status::text AS "status!",
                     cover_template_id, visibility::text AS "visibility!", requires_approval"#,
        id,
        edit.title,
        edit.category,
        edit.description.is_some(),
        edit.description.flatten(),
        edit.location_name.is_some(),
        edit.location_name.flatten(),
        edit.starts_at,
        edit.ends_at.is_some(),
        edit.ends_at.flatten(),
        edit.timezone,
        edit.capacity.is_some(),
        edit.capacity.flatten(),
        edit.max_plus_ones,
        edit.cover_template_id.is_some(),
        edit.cover_template_id.flatten(),
        edit.visibility,
        edit.requires_approval
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::from_sqlx)?;

    row.into_record()
}
