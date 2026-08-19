use std::fmt;

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportSubject {
    Message,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportReason {
    Spam,
    Harassment,
    HateSpeech,
    Nudity,
    Violence,
    Impersonation,
    Other,
}

pub const MAX_REPORT_DETAIL_LENGTH: usize = 1_000;

