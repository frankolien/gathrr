use std::time::Duration;

use anyhow::Context;
use gathr_common::{telemetry, Config};
use gathr_infra_push::{Apns, Notification, PushError};
use gathr_worker::{drain_once, Dispatcher};
use time::OffsetDateTime;

const POLL_INTERVAL: Duration = Duration::from_secs(30);

struct ApnsDispatcher(Option<Apns>);

impl Dispatcher for ApnsDispatcher {
    async fn deliver(&self, notification: Notification) -> Result<(), String> {
        let Some(apns) = self.0.as_ref() else {
            tracing::info!(
                title = %notification.title,
                device = %&notification.device_token[..8.min(notification.device_token.len())],
                "push is not configured; logging the notification instead of sending it"
            );
            return Ok(());
        };

        apns.send(&notification).await.map_err(|error| match error {
            PushError::Rejected { reason, .. } => reason,
            other => other.to_string(),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init();

    let config = Config::from_env().context("configuration is incomplete")?;
    let db = gathr_infra_db::connect(&config.database_url, 8)
        .await
        .context("could not reach postgres")?;

    let apns = match Apns::new(
        config.apns_team_id.clone(),
        config.apns_key_id.clone(),
        &config.apns_private_key,
        config.apns_topic.clone(),
    ) {
        Some(built) => Some(built.context("the apns signing key could not be loaded")?),
        None => {
            tracing::warn!("apns is not configured; reminders will be logged, not delivered");
            None
        }
    };

    let dispatcher = ApnsDispatcher(apns);
    tracing::info!(
        interval_seconds = POLL_INTERVAL.as_secs(),
        "gathr worker started"
    );

    loop {
        match drain_once(&db, &dispatcher, OffsetDateTime::now_utc()).await {
            Ok(sweep) if sweep.claimed > 0 => tracing::info!(
                claimed = sweep.claimed,
                sent = sweep.sent,
                failed = sweep.failed,
                notifications = sweep.notifications,
                "reminder sweep finished"
            ),
            Ok(_) => {}
            Err(error) => tracing::error!(%error, "reminder sweep failed"),
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
