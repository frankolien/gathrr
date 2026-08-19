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

