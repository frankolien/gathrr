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

