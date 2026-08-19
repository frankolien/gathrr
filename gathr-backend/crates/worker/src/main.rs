use std::time::Duration;

use anyhow::Context;
use gathr_common::{telemetry, Config};
use gathr_infra_push::{Apns, Notification, PushError};
use gathr_worker::{drain_once, Dispatcher};
use time::OffsetDateTime;

const POLL_INTERVAL: Duration = Duration::from_secs(30);

struct ApnsDispatcher(Option<Apns>);

