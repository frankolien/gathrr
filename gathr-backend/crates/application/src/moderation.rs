use gathr_domain::report::sanitize_detail;
use gathr_domain::{DomainError, ReportReason, ReportSubject};
use gathr_infra_db::{moderation, users, Db};
use uuid::Uuid;

use crate::error::AppError;
use crate::messages;

#[derive(Debug, Clone)]
pub struct NewReport<'a> {
    pub reporter_id: Uuid,
    pub subject: &'a str,
    pub subject_id: Uuid,
    pub reason: &'a str,
    pub detail: Option<&'a str>,
}

pub async fn report(db: &Db, input: NewReport<'_>) -> Result<Uuid, AppError> {
    let subject = ReportSubject::parse(input.subject)?;
    let reason = ReportReason::parse(input.reason)?;
    let detail = sanitize_detail(input.detail)?;

    let event_id = match subject {
        ReportSubject::User => {
            if input.subject_id == input.reporter_id {
                return Err(DomainError::SelfReport.into());
            }
            users::find(db, input.subject_id)
                .await?
                .ok_or(AppError::NotFound)?;
            None
        }
        ReportSubject::Message => {
            let message = messages::find_readable(db, input.subject_id, input.reporter_id).await?;
            if message.sender_id == Some(input.reporter_id) {
                return Err(DomainError::SelfReport.into());
            }
            Some(message.event_id)
        }
    };

    Ok(moderation::file_report(
        db,
        input.reporter_id,
        subject.as_str(),
        input.subject_id,
        event_id,
        reason.as_str(),
        detail.as_deref(),
    )
    .await?)
}

pub async fn block(db: &Db, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), AppError> {
    if blocker_id == blocked_id {
        return Err(DomainError::SelfBlock.into());
    }
    users::find(db, blocked_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(moderation::block(db, blocker_id, blocked_id).await?)
}

pub async fn unblock(db: &Db, blocker_id: Uuid, blocked_id: Uuid) -> Result<(), AppError> {
    moderation::unblock(db, blocker_id, blocked_id)
        .await?
        .then_some(())
        .ok_or(AppError::NotFound)
}

pub async fn blocked(db: &Db, blocker_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    Ok(moderation::blocked_by(db, blocker_id).await?)
}
