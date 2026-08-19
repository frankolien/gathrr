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

pub fn headline(kind: &str, title: &str) -> (String, String) {
    match kind {
        DAY_BEFORE => (
            title.to_owned(),
            "Tomorrow. Tap to see the details and who's coming.".to_owned(),
        ),
        HOURS_BEFORE => (title.to_owned(), "Starts in about 2 hours.".to_owned()),
        _ => (
            title.to_owned(),
            "There's an update to this event.".to_owned(),
        ),
    }
}

pub async fn drain_once(
    db: &Db,
    dispatcher: &impl Dispatcher,
    now: OffsetDateTime,
) -> Result<Sweep, DbError> {
    let mut tx = db.begin().await.map_err(DbError::from_sqlx)?;
    let due = reminders::claim_due(&mut tx, now, BATCH_SIZE).await?;
    tx.commit().await.map_err(DbError::from_sqlx)?;

    let mut sweep = Sweep {
        claimed: due.len(),
        ..Sweep::default()
    };

    for job in due {
        match deliver_one(db, dispatcher, job.event_id, &job.kind).await {
            Ok(delivered) => {
                sweep.notifications += delivered;
                sweep.sent += 1;
                reminders::mark_sent(db, job.id).await?;
            }
            Err(reason) => {
                tracing::warn!(job = %job.id, kind = %job.kind, %reason, "reminder could not be delivered");
                sweep.failed += 1;
                reminders::mark_failed(db, job.id, &reason, MAX_ATTEMPTS).await?;
            }
        }
    }

    Ok(sweep)
}

async fn deliver_one(
    db: &Db,
    dispatcher: &impl Dispatcher,
    event_id: Uuid,
    kind: &str,
) -> Result<usize, String> {
    let event = events::find(db, event_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "the event no longer exists".to_owned())?;

    gathr_application::activity::record_reminder(db, event_id).await;

    let audience = devices::audience_for_event(db, event_id)
        .await
        .map_err(|error| error.to_string())?;

    let (title, body) = headline(kind, &event.title);
    let mut delivered = 0;

    for device in audience {
        let notification = Notification {
            device_token: device.apns_token,
            environment: device.environment,
            title: title.clone(),
            body: body.clone(),
            thread_id: event_id.to_string(),
        };

        match dispatcher.deliver(notification).await {
            Ok(()) => delivered += 1,
            Err(reason) => {
                tracing::warn!(device = %device.id, %reason, "a device did not accept the push")
            }
        }
    }

    Ok(delivered)
}
