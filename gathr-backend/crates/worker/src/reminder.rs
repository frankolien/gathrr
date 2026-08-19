use std::future::Future;

use gathr_application::notifications::{DAY_BEFORE, HOURS_BEFORE, MAX_ATTEMPTS};
use gathr_infra_db::{devices, events, reminders, Db, DbError};
use gathr_infra_push::Notification;
use time::OffsetDateTime;
use uuid::Uuid;

pub const BATCH_SIZE: i64 = 50;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sweep {
    pub claimed: usize,
    pub sent: usize,
    pub failed: usize,
    pub notifications: usize,
}

pub trait Dispatcher {
    fn deliver(
        &self,
        notification: Notification,
    ) -> impl Future<Output = Result<(), String>> + Send;
}

