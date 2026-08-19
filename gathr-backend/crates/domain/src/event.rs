use std::fmt;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Draft,
    Published,
    Ongoing,
    Ended,
    Cancelled,
}

impl EventStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Ongoing => "ongoing",
            Self::Ended => "ended",
            Self::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventSchedule {
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
}

impl EventSchedule {
    pub fn observed_status(&self, stored: EventStatus, now: OffsetDateTime) -> EventStatus {
        if stored != EventStatus::Published && stored != EventStatus::Ongoing {
            return stored;
        }
        match self.ends_at {
            Some(ends_at) if now >= ends_at => EventStatus::Ended,
            _ if now >= self.starts_at => EventStatus::Ongoing,
            _ => EventStatus::Published,
        }
    }
}

pub fn publish(
    current: EventStatus,
    title: &str,
    schedule: Option<EventSchedule>,
) -> Result<EventStatus, DomainError> {
    if current != EventStatus::Draft {
        return Err(DomainError::IllegalEventTransition {
            from: current,
            to: EventStatus::Published,
        });
    }
    if title.trim().is_empty() {
        return Err(DomainError::EventIncomplete { field: "a title" });
    }
    if schedule.is_none() {
        return Err(DomainError::EventIncomplete {
            field: "a start time",
        });
    }
    Ok(EventStatus::Published)
}

