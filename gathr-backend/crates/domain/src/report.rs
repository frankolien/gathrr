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

impl ReportSubject {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "message" => Ok(Self::Message),
            "user" => Ok(Self::User),
            _ => Err(DomainError::ReportSubjectUnknown),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::User => "user",
        }
    }
}

impl ReportReason {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "spam" => Ok(Self::Spam),
            "harassment" => Ok(Self::Harassment),
            "hate_speech" => Ok(Self::HateSpeech),
            "nudity" => Ok(Self::Nudity),
            "violence" => Ok(Self::Violence),
            "impersonation" => Ok(Self::Impersonation),
            "other" => Ok(Self::Other),
            _ => Err(DomainError::ReportReasonUnknown),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spam => "spam",
            Self::Harassment => "harassment",
            Self::HateSpeech => "hate_speech",
            Self::Nudity => "nudity",
            Self::Violence => "violence",
            Self::Impersonation => "impersonation",
            Self::Other => "other",
        }
    }
}

pub fn sanitize_detail(detail: Option<&str>) -> Result<Option<String>, DomainError> {
    let Some(detail) = detail else {
        return Ok(None);
    };
    let trimmed = detail.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_REPORT_DETAIL_LENGTH {
        return Err(DomainError::ReportDetailTooLong);
    }
    Ok(Some(trimmed.to_owned()))
}

impl fmt::Display for ReportSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for ReportReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
