use time::OffsetDateTime;

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InviteTerms {
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<OffsetDateTime>,
}

impl InviteTerms {
    pub fn multi_use() -> Self {
        Self {
            max_uses: None,
            uses: 0,
            expires_at: None,
        }
    }

    pub fn single_use() -> Self {
        Self {
            max_uses: Some(1),
            uses: 0,
            expires_at: None,
        }
    }

    pub fn guard_redeemable(&self, now: OffsetDateTime) -> Result<(), DomainError> {
        if let Some(expires_at) = self.expires_at {
            if now >= expires_at {
                return Err(DomainError::InviteExpired);
            }
        }
        if let Some(max_uses) = self.max_uses {
            if self.uses >= max_uses {
                return Err(DomainError::InviteExhausted);
            }
        }
        Ok(())
    }

    pub fn remaining_uses(&self) -> Option<i32> {
        self.max_uses.map(|max| (max - self.uses).max(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    fn now() -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + Duration::hours(100)
    }

    #[test]
    fn an_open_invite_is_always_redeemable() {
        assert_eq!(InviteTerms::multi_use().guard_redeemable(now()), Ok(()));
    }

    #[test]
    fn a_single_use_invite_is_spent_after_one_use() {
        let mut terms = InviteTerms::single_use();
        assert_eq!(terms.guard_redeemable(now()), Ok(()));
        terms.uses = 1;
        assert_eq!(
            terms.guard_redeemable(now()),
            Err(DomainError::InviteExhausted)
        );
    }

    #[test]
    fn expiry_is_exclusive_at_the_boundary() {
        let terms = InviteTerms {
            expires_at: Some(now()),
            ..InviteTerms::multi_use()
        };
        assert_eq!(
            terms.guard_redeemable(now()),
            Err(DomainError::InviteExpired)
        );
        assert_eq!(terms.guard_redeemable(now() - Duration::seconds(1)), Ok(()));
    }

    #[test]
    fn expiry_is_reported_before_exhaustion() {
        let terms = InviteTerms {
            max_uses: Some(1),
            uses: 1,
            expires_at: Some(now() - Duration::hours(1)),
        };
        assert_eq!(
            terms.guard_redeemable(now()),
            Err(DomainError::InviteExpired)
        );
    }

    #[test]
    fn remaining_uses_never_reports_a_negative() {
        let terms = InviteTerms {
            max_uses: Some(1),
            uses: 3,
            expires_at: None,
        };
        assert_eq!(terms.remaining_uses(), Some(0));
        assert_eq!(InviteTerms::multi_use().remaining_uses(), None);
    }
}
