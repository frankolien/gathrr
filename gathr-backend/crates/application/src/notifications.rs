use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use gathr_infra_db::{devices, events as event_rows, reminders, Db};

use crate::error::AppError;
use crate::events;

pub const DAY_BEFORE: &str = "day_before";
pub const HOURS_BEFORE: &str = "hours_before";
pub const MAX_ATTEMPTS: i32 = 5;

