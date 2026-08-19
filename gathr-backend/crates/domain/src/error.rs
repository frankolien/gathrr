use thiserror::Error;

use crate::event::EventStatus;
use crate::rsvp::RsvpStatus;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("event is at capacity")]
    CapacityExceeded { seats_short: i32 },

    #[error("requested {requested} plus-ones but the event allows {max}")]
    PlusOnesExceeded { requested: i32, max: i32 },

    #[error("plus-one count cannot be negative")]
    PlusOnesNegative,

    #[error("an rsvp cannot move from {from} to {to}")]
    IllegalRsvpTransition { from: RsvpStatus, to: RsvpStatus },

    #[error("an event cannot move from {from} to {to}")]
    IllegalEventTransition { from: EventStatus, to: EventStatus },

    #[error("this event has been cancelled")]
    EventCancelled,

    #[error("this event has already ended")]
    EventEnded,

    #[error("an event cannot be published without {field}")]
    EventIncomplete { field: &'static str },

    #[error("this invite has expired")]
    InviteExpired,

    #[error("this invite has been used up")]
    InviteExhausted,

    #[error("this invite code is not valid")]
    InviteCodeInvalid,

    #[error("a message cannot be empty")]
    MessageEmpty,

    #[error(
        "a message cannot be longer than {} characters",
        crate::message::MAX_MESSAGE_LENGTH
    )]
    MessageTooLong,
}

