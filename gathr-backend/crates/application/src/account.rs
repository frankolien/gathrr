use gathr_infra_db::account::{
    self, ExportedEvent, ExportedIdentity, ExportedMessage, ExportedRsvp,
};
use gathr_infra_db::{messages as message_rows, moderation, users, Db, UserRecord};
use uuid::Uuid;

use crate::error::AppError;
use crate::events;

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
    pub events_cancelled: usize,
    pub messages_redacted: u64,
}

