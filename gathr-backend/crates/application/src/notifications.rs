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

pub async fn register_device(
    db: &Db,
    user_id: Uuid,
    apns_token: &str,
    environment: Option<&str>,
) -> Result<Uuid, AppError> {
    let apns_token = apns_token.trim();
    let plausible = apns_token.len() >= 32 && apns_token.chars().all(|c| c.is_ascii_hexdigit());
    if !plausible {
        return Err(AppError::Validation(
            "that does not look like an apns device token".to_owned(),
        ));
    }

    let environment = match environment {
        Some("production") => "production",
        None | Some("sandbox") => "sandbox",
        Some(other) => {
            return Err(AppError::Validation(format!(
                "{other} is not an apns environment"
            )))
        }
    };

    Ok(devices::upsert(db, user_id, apns_token, environment)
        .await?
        .id)
}

pub async fn forget_device(db: &Db, user_id: Uuid, device_id: Uuid) -> Result<(), AppError> {
    devices::remove(db, device_id, user_id)
        .await?
        .then_some(())
        .ok_or(AppError::NotFound)
}

pub async fn set_mute(db: &Db, user_id: Uuid, event_id: Uuid, muted: bool) -> Result<(), AppError> {
    events::detail(db, event_id).await?;
    Ok(devices::set_mute(db, user_id, event_id, muted).await?)
}
