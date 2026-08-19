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

    #[error("a report is about a message or a user, nothing else")]
    ReportSubjectUnknown,

    #[error("that is not a reason anyone can act on")]
    ReportReasonUnknown,

    #[error(
        "report detail cannot be longer than {} characters",
        crate::report::MAX_REPORT_DETAIL_LENGTH
    )]
    ReportDetailTooLong,

    #[error("you cannot block yourself")]
    SelfBlock,

    #[error("you cannot report yourself")]
    SelfReport,
}

impl DomainError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::CapacityExceeded { .. } => "capacity_exceeded",
            Self::PlusOnesExceeded { .. } | Self::PlusOnesNegative => "plus_ones_exceeded",
            Self::IllegalRsvpTransition { .. } | Self::IllegalEventTransition { .. } => {
                "validation_failed"
            }
            Self::EventCancelled => "event_cancelled",
            Self::EventEnded => "event_ended",
            Self::EventIncomplete { .. } => "validation_failed",
            Self::InviteExpired => "invite_expired",
            Self::InviteExhausted => "invite_exhausted",
            Self::InviteCodeInvalid => "invite_invalid",
            Self::MessageEmpty | Self::MessageTooLong => "message_invalid",
            Self::ReportSubjectUnknown | Self::ReportReasonUnknown | Self::ReportDetailTooLong => {
                "report_invalid"
            }
            Self::SelfBlock | Self::SelfReport => "self_target",
        }
    }
}
