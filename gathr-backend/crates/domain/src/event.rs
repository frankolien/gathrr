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

pub fn cancel(current: EventStatus) -> Result<EventStatus, DomainError> {
    match current {
        EventStatus::Cancelled => Err(DomainError::EventCancelled),
        EventStatus::Ended => Err(DomainError::EventEnded),
        _ => Ok(EventStatus::Cancelled),
    }
}

pub fn guard_accepts_rsvps(current: EventStatus) -> Result<(), DomainError> {
    match current {
        EventStatus::Cancelled => Err(DomainError::EventCancelled),
        EventStatus::Ended => Err(DomainError::EventEnded),
        EventStatus::Draft => Err(DomainError::IllegalEventTransition {
            from: current,
            to: EventStatus::Published,
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    fn at(offset_hours: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + Duration::hours(offset_hours)
    }

    fn schedule(start: i64, end: Option<i64>) -> EventSchedule {
        EventSchedule {
            starts_at: at(start),
            ends_at: end.map(at),
        }
    }

    #[test]
    fn a_complete_draft_publishes() {
        assert_eq!(
            publish(EventStatus::Draft, "Amara's 26th", Some(schedule(10, None))),
            Ok(EventStatus::Published)
        );
    }

    #[test]
    fn a_draft_without_a_title_or_start_time_cannot_publish() {
        assert_eq!(
            publish(EventStatus::Draft, "   ", Some(schedule(10, None))),
            Err(DomainError::EventIncomplete { field: "a title" })
        );
        assert_eq!(
            publish(EventStatus::Draft, "Amara's 26th", None),
            Err(DomainError::EventIncomplete {
                field: "a start time"
            })
        );
    }

    #[test]
    fn only_a_draft_can_publish() {
        assert!(publish(EventStatus::Published, "x", Some(schedule(10, None))).is_err());
        assert!(publish(EventStatus::Cancelled, "x", Some(schedule(10, None))).is_err());
    }

    #[test]
    fn terminal_events_cannot_be_cancelled_again() {
        assert_eq!(cancel(EventStatus::Published), Ok(EventStatus::Cancelled));
        assert_eq!(cancel(EventStatus::Draft), Ok(EventStatus::Cancelled));
        assert_eq!(
            cancel(EventStatus::Cancelled),
            Err(DomainError::EventCancelled)
        );
        assert_eq!(cancel(EventStatus::Ended), Err(DomainError::EventEnded));
    }

    #[test]
    fn lifecycle_advances_with_the_clock_without_a_job() {
        let s = schedule(10, Some(14));
        assert_eq!(
            s.observed_status(EventStatus::Published, at(9)),
            EventStatus::Published
        );
        assert_eq!(
            s.observed_status(EventStatus::Published, at(10)),
            EventStatus::Ongoing
        );
        assert_eq!(
            s.observed_status(EventStatus::Published, at(14)),
            EventStatus::Ended
        );
    }

    #[test]
    fn an_event_without_an_end_time_never_auto_ends() {
        let s = schedule(10, None);
        assert_eq!(
            s.observed_status(EventStatus::Published, at(9_999)),
            EventStatus::Ongoing
        );
    }

    #[test]
    fn a_cancelled_event_is_never_reanimated_by_the_clock() {
        let s = schedule(10, Some(14));
        assert_eq!(
            s.observed_status(EventStatus::Cancelled, at(12)),
            EventStatus::Cancelled
        );
    }

    #[test]
    fn rsvps_are_refused_outside_the_live_window() {
        assert!(guard_accepts_rsvps(EventStatus::Published).is_ok());
        assert!(guard_accepts_rsvps(EventStatus::Ongoing).is_ok());
        assert_eq!(
            guard_accepts_rsvps(EventStatus::Cancelled),
            Err(DomainError::EventCancelled)
        );
        assert_eq!(
            guard_accepts_rsvps(EventStatus::Ended),
            Err(DomainError::EventEnded)
        );
        assert!(guard_accepts_rsvps(EventStatus::Draft).is_err());
    }
}
