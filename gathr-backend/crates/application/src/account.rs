use gathr_infra_db::account::{
    self, ExportedEvent, ExportedIdentity, ExportedMessage, ExportedRsvp,
};
use gathr_infra_db::{
    hosts as host_rows, messages as message_rows, moderation, users, Db, UserRecord,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::events;
use crate::hosts;

#[derive(Debug, Clone)]
pub struct Export {
    pub account: UserRecord,
    pub hosted_events: Vec<ExportedEvent>,
    pub rsvps: Vec<ExportedRsvp>,
    pub messages: Vec<ExportedMessage>,
    pub identities: Vec<ExportedIdentity>,
    pub blocked_user_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erasure {
    pub events_handed_over: usize,
    pub events_cancelled: usize,
    pub messages_redacted: u64,
}

pub async fn export(db: &Db, user_id: Uuid) -> Result<Export, AppError> {
    let account = users::find(db, user_id).await?.ok_or(AppError::NotFound)?;

    Ok(Export {
        account,
        hosted_events: account::hosted_events(db, user_id).await?,
        rsvps: account::rsvps(db, user_id).await?,
        messages: account::messages(db, user_id).await?,
        identities: account::identities(db, user_id).await?,
        blocked_user_ids: moderation::blocked_by(db, user_id).await?,
    })
}

pub async fn erase(db: &Db, user_id: Uuid) -> Result<Erasure, AppError> {
    users::find(db, user_id).await?.ok_or(AppError::NotFound)?;

    let owned = host_rows::events_needing_a_new_owner(db, user_id).await?;
    let mut events_handed_over = 0;
    let mut events_cancelled = 0;
    for event_id in owned {
        if hosts::hand_over(db, event_id, user_id).await?.is_some() {
            events_handed_over += 1;
        } else if events::cancel(db, event_id, user_id).await.is_ok() {
            events_cancelled += 1;
        }
    }

    let messages_redacted = message_rows::redact_by_sender(db, user_id).await?;

    account::erase(db, user_id)
        .await?
        .then_some(())
        .ok_or(AppError::NotFound)?;

    Ok(Erasure {
        events_handed_over,
        events_cancelled,
        messages_redacted,
    })
}
