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

pub fn promote_from_waitlist(
    current: RsvpStatus,
    plus_ones: i32,
    capacity: CapacityContext,
) -> Result<RsvpOutcome, DomainError> {
    if current != RsvpStatus::Waitlisted {
        return Err(DomainError::IllegalRsvpTransition {
            from: current,
            to: RsvpStatus::Going,
        });
    }
    match capacity.evaluate(plus_ones)? {
        Admission::Admitted => Ok(RsvpOutcome {
            status: RsvpStatus::Going,
            plus_ones,
            entered_waitlist: false,
        }),
        Admission::Full { seats_short } => Err(DomainError::CapacityExceeded { seats_short }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> CapacityContext {
        CapacityContext::unlimited()
    }

    fn full() -> CapacityContext {
        CapacityContext {
            capacity: Some(2),
            seats_held_excluding_actor: 2,
            max_plus_ones: 2,
        }
    }

    fn request(status: RsvpStatus) -> RsvpRequest {
        RsvpRequest {
            status,
            plus_ones: 0,
            accept_waitlist: false,
        }
    }

    #[test]
    fn a_first_time_guest_can_go() {
        let outcome = submit(None, request(RsvpStatus::Going), open()).unwrap();
        assert_eq!(outcome.status, RsvpStatus::Going);
    }

    #[test]
    fn the_three_guest_statuses_are_freely_interchangeable() {
        for from in [RsvpStatus::Going, RsvpStatus::Maybe, RsvpStatus::Declined] {
            for to in [RsvpStatus::Going, RsvpStatus::Maybe, RsvpStatus::Declined] {
                let outcome = submit(Some(from), request(to), open()).unwrap();
                assert_eq!(outcome.status, to, "{from} -> {to} should be allowed");
            }
        }
    }

    #[test]
    fn a_guest_cannot_place_themselves_on_the_waitlist() {
        let error = submit(None, request(RsvpStatus::Waitlisted), open()).unwrap_err();
        assert!(matches!(error, DomainError::IllegalRsvpTransition { .. }));
    }

    #[test]
    fn a_guest_cannot_return_themselves_to_invited() {
        let error = submit(
            Some(RsvpStatus::Going),
            request(RsvpStatus::Invited),
            open(),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::IllegalRsvpTransition { .. }));
    }

    #[test]
    fn a_full_event_rejects_going_rather_than_silently_waitlisting() {
        let error = submit(None, request(RsvpStatus::Going), full()).unwrap_err();
        assert_eq!(error, DomainError::CapacityExceeded { seats_short: 1 });
    }

    #[test]
    fn a_full_event_waitlists_only_when_the_guest_opted_in() {
        let outcome = submit(
            None,
            RsvpRequest {
                status: RsvpStatus::Going,
                plus_ones: 0,
                accept_waitlist: true,
            },
            full(),
        )
        .unwrap();
        assert_eq!(outcome.status, RsvpStatus::Waitlisted);
        assert!(outcome.entered_waitlist);
    }

    #[test]
    fn re_confirming_a_waitlist_place_does_not_move_the_guest_to_the_back() {
        let outcome = submit(
            Some(RsvpStatus::Waitlisted),
            RsvpRequest {
                status: RsvpStatus::Going,
                plus_ones: 0,
                accept_waitlist: true,
            },
            full(),
        )
        .unwrap();
        assert!(!outcome.entered_waitlist);
    }

    #[test]
    fn declining_is_always_possible_even_when_the_event_is_full() {
        let outcome = submit(
            Some(RsvpStatus::Waitlisted),
            request(RsvpStatus::Declined),
            full(),
        )
        .unwrap();
        assert_eq!(outcome.status, RsvpStatus::Declined);
    }

    #[test]
    fn promotion_requires_a_waitlisted_guest_and_a_free_seat() {
        let freed = CapacityContext {
            capacity: Some(2),
            seats_held_excluding_actor: 1,
            max_plus_ones: 2,
        };
        assert_eq!(
            promote_from_waitlist(RsvpStatus::Waitlisted, 0, freed)
                .unwrap()
                .status,
            RsvpStatus::Going
        );
        assert_eq!(
            promote_from_waitlist(RsvpStatus::Waitlisted, 0, full()).unwrap_err(),
            DomainError::CapacityExceeded { seats_short: 1 }
        );
        assert!(promote_from_waitlist(RsvpStatus::Declined, 0, freed).is_err());
    }

    #[test]
    fn only_going_holds_seats() {
        assert!(RsvpStatus::Going.holds_seats());
        for status in [
            RsvpStatus::Invited,
            RsvpStatus::Maybe,
            RsvpStatus::Declined,
            RsvpStatus::Waitlisted,
        ] {
            assert!(!status.holds_seats());
        }
    }
}
