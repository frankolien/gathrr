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

pub fn may_post(right: PostingRight, is_host: bool, is_participant: bool) -> bool {
    match right {
        PostingRight::HostOnly => is_host,
        PostingRight::Participant => is_host || is_participant,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_not_content() {
        assert_eq!(sanitize("   \n\t "), Err(DomainError::MessageEmpty));
        assert_eq!(sanitize("  running late  ").unwrap(), "running late");
    }

    #[test]
    fn the_length_limit_counts_characters_not_bytes() {
        let emoji = "🎉".repeat(MAX_MESSAGE_LENGTH);
        assert!(
            sanitize(&emoji).is_ok(),
            "a message at the limit must be accepted however many bytes it takes"
        );
        assert_eq!(
            sanitize(&"a".repeat(MAX_MESSAGE_LENGTH + 1)),
            Err(DomainError::MessageTooLong)
        );
    }

    #[test]
    fn announcements_are_host_only_while_chat_admits_participants() {
        assert!(may_post(PostingRight::HostOnly, true, false));
        assert!(!may_post(PostingRight::HostOnly, false, true));

        assert!(may_post(PostingRight::Participant, false, true));
        assert!(may_post(PostingRight::Participant, true, false));
        assert!(
            !may_post(PostingRight::Participant, false, false),
            "a stranger must not be able to post into someone else's event"
        );
    }
}
