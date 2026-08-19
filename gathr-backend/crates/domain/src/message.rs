use crate::error::DomainError;

pub const MAX_MESSAGE_LENGTH: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostingRight {
    HostOnly,
    Participant,
}

pub fn sanitize(body: &str) -> Result<String, DomainError> {
    let trimmed = body.trim();

    if trimmed.is_empty() {
        return Err(DomainError::MessageEmpty);
    }
    if trimmed.chars().count() > MAX_MESSAGE_LENGTH {
        return Err(DomainError::MessageTooLong);
    }

    Ok(trimmed.to_owned())
}

