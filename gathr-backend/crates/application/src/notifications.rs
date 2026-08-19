use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use gathr_infra_db::{devices, events as event_rows, reminders, Db};

use crate::error::AppError;
use crate::events;

pub const DAY_BEFORE: &str = "day_before";
pub const HOURS_BEFORE: &str = "hours_before";
pub const MAX_ATTEMPTS: i32 = 5;

pub fn schedule_for(
    starts_at: OffsetDateTime,
    now: OffsetDateTime,
) -> Vec<(&'static str, OffsetDateTime)> {
    [
        (DAY_BEFORE, starts_at - Duration::hours(24)),
        (HOURS_BEFORE, starts_at - Duration::hours(2)),
    ]
    .into_iter()
    .filter(|(_, run_at)| *run_at > now)
    .collect()
}

pub async fn plan_reminders(db: &Db, event_id: Uuid) -> Result<(), AppError> {
    let event = event_rows::find(db, event_id)
        .await?
        .ok_or(AppError::NotFound)?;

    for (kind, run_at) in schedule_for(event.starts_at, OffsetDateTime::now_utc()) {
        reminders::schedule(db, event_id, kind, run_at).await?;
    }
    Ok(())
}

pub async fn cancel_reminders(db: &Db, event_id: Uuid) -> Result<(), AppError> {
    Ok(reminders::cancel_for_event(db, event_id).await?)
}

