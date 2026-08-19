use std::fmt;

use serde::{Deserialize, Serialize};

use crate::capacity::{Admission, CapacityContext};
use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsvpStatus {
    Invited,
    Going,
    Maybe,
    Declined,
    Waitlisted,
}

impl RsvpStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Invited => "invited",
            Self::Going => "going",
            Self::Maybe => "maybe",
            Self::Declined => "declined",
            Self::Waitlisted => "waitlisted",
        }
    }

    pub fn is_guest_selectable(self) -> bool {
        matches!(self, Self::Going | Self::Maybe | Self::Declined)
    }

    pub fn holds_seats(self) -> bool {
        matches!(self, Self::Going)
    }
}

impl fmt::Display for RsvpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsvpRequest {
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub accept_waitlist: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsvpOutcome {
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub entered_waitlist: bool,
}

pub fn submit(
    current: Option<RsvpStatus>,
    request: RsvpRequest,
    capacity: CapacityContext,
) -> Result<RsvpOutcome, DomainError> {
    if !request.status.is_guest_selectable() {
        return Err(DomainError::IllegalRsvpTransition {
            from: current.unwrap_or(RsvpStatus::Invited),
            to: request.status,
        });
    }

    capacity.validate_plus_ones(request.plus_ones)?;

    if request.status != RsvpStatus::Going {
        return Ok(RsvpOutcome {
            status: request.status,
            plus_ones: request.plus_ones,
            entered_waitlist: false,
        });
    }

    match capacity.evaluate(request.plus_ones)? {
        Admission::Admitted => Ok(RsvpOutcome {
            status: RsvpStatus::Going,
            plus_ones: request.plus_ones,
            entered_waitlist: false,
        }),
        Admission::Full { .. } if request.accept_waitlist => Ok(RsvpOutcome {
            status: RsvpStatus::Waitlisted,
            plus_ones: request.plus_ones,
            entered_waitlist: current != Some(RsvpStatus::Waitlisted),
        }),
        Admission::Full { seats_short } => Err(DomainError::CapacityExceeded { seats_short }),
    }
}

