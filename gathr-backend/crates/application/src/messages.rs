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
    pub sender_id: Option<Uuid>,
    pub sender_display_name: Option<String>,
    pub seq: i64,
    pub body: String,
    pub redacted: bool,
    pub created_at: OffsetDateTime,
}

impl From<messages::MessageRecord> for MessageView {
    fn from(record: messages::MessageRecord) -> Self {
        Self {
            id: record.id,
            event_id: record.event_id,
            sender_id: record.sender_id,
            sender_display_name: record.sender_display_name,
            seq: record.seq,
            body: record.body,
            redacted: record.redacted,
            created_at: record.created_at,
        }
    }
}

pub async fn post(
    db: &Db,
    event_id: Uuid,
    sender_id: Uuid,
    body: &str,
    right: PostingRight,
) -> Result<MessageView, AppError> {
    let body = sanitize(body)?;
    authorize_post(db, event_id, sender_id, right).await?;

    let mut tx = db
        .begin()
        .await
        .map_err(gathr_infra_db::DbError::from_sqlx)?;
    let seq = messages::allocate_seq(&mut tx, event_id).await?;
    let record = messages::insert(&mut tx, event_id, sender_id, seq, &body).await?;
    tx.commit()
        .await
        .map_err(gathr_infra_db::DbError::from_sqlx)?;
    crate::activity::record_message(db, event_id, sender_id).await;

    Ok(record.into())
}

pub async fn page(
    db: &Db,
    event_id: Uuid,
    reader_id: Uuid,
    after_seq: i64,
    limit: Option<i64>,
) -> Result<Vec<MessageView>, AppError> {
    authorize_read(db, event_id, reader_id).await?;

    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let records = messages::page(db, event_id, reader_id, after_seq.max(0), limit).await?;
    Ok(records.into_iter().map(MessageView::from).collect())
}

pub async fn latest_seq(db: &Db, event_id: Uuid, reader_id: Uuid) -> Result<i64, AppError> {
    authorize_read(db, event_id, reader_id).await?;
    Ok(messages::latest_seq(db, event_id).await?)
}

pub async fn authorize_read(db: &Db, event_id: Uuid, reader_id: Uuid) -> Result<(), AppError> {
    let is_host = events::can_manage(db, event_id, reader_id).await?;
    let is_participant = rsvps::find(db, event_id, reader_id).await?.is_some();

    if is_host || is_participant {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

async fn authorize_post(
    db: &Db,
    event_id: Uuid,
    sender_id: Uuid,
    right: PostingRight,
) -> Result<(), AppError> {
    let is_host = events::can_manage(db, event_id, sender_id).await?;
    let is_participant = rsvps::find(db, event_id, sender_id).await?.is_some();

    if may_post(right, is_host, is_participant) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub async fn find_readable(
    db: &Db,
    message_id: Uuid,
    reader_id: Uuid,
) -> Result<MessageView, AppError> {
    let record = messages::find(db, message_id)
        .await?
        .ok_or(AppError::NotFound)?;
    authorize_read(db, record.event_id, reader_id).await?;
    Ok(record.into())
}
