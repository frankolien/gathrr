use crate::error::DomainError;

pub const DEFAULT_MAX_PLUS_ONES: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityContext {
    pub capacity: Option<i32>,
    pub seats_held_excluding_actor: i32,
    pub max_plus_ones: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    Admitted,
    Full { seats_short: i32 },
}

pub fn seats_needed(plus_ones: i32) -> i32 {
    plus_ones + 1
}

impl CapacityContext {
    pub fn unlimited() -> Self {
        Self {
            capacity: None,
            seats_held_excluding_actor: 0,
            max_plus_ones: DEFAULT_MAX_PLUS_ONES,
        }
    }

    pub fn validate_plus_ones(&self, plus_ones: i32) -> Result<(), DomainError> {
        if plus_ones < 0 {
            return Err(DomainError::PlusOnesNegative);
        }
        if plus_ones > self.max_plus_ones {
            return Err(DomainError::PlusOnesExceeded {
                requested: plus_ones,
                max: self.max_plus_ones,
            });
        }
        Ok(())
    }

    pub fn evaluate(&self, plus_ones: i32) -> Result<Admission, DomainError> {
        self.validate_plus_ones(plus_ones)?;
        let Some(capacity) = self.capacity else {
            return Ok(Admission::Admitted);
        };
        let required = self.seats_held_excluding_actor + seats_needed(plus_ones);
        if required <= capacity {
            Ok(Admission::Admitted)
        } else {
            Ok(Admission::Full {
                seats_short: required - capacity,
            })
        }
    }

    pub fn remaining_seats(&self) -> Option<i32> {
        self.capacity
            .map(|capacity| (capacity - self.seats_held_excluding_actor).max(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(capacity: Option<i32>, held: i32) -> CapacityContext {
        CapacityContext {
            capacity,
            seats_held_excluding_actor: held,
            max_plus_ones: DEFAULT_MAX_PLUS_ONES,
        }
    }

    #[test]
    fn a_going_rsvp_consumes_one_seat_plus_its_plus_ones() {
        assert_eq!(seats_needed(0), 1);
        assert_eq!(seats_needed(2), 3);
    }

    #[test]
    fn null_capacity_always_admits() {
        assert_eq!(context(None, 9_999).evaluate(2), Ok(Admission::Admitted));
    }

    #[test]
    fn the_last_seat_is_admitted() {
        assert_eq!(context(Some(10), 9).evaluate(0), Ok(Admission::Admitted));
    }

    #[test]
    fn one_seat_beyond_capacity_is_full() {
        assert_eq!(
            context(Some(10), 10).evaluate(0),
            Ok(Admission::Full { seats_short: 1 })
        );
    }

    #[test]
    fn plus_ones_are_counted_against_capacity() {
        assert_eq!(
            context(Some(10), 8).evaluate(2),
            Ok(Admission::Full { seats_short: 1 })
        );
        assert_eq!(context(Some(10), 8).evaluate(1), Ok(Admission::Admitted));
    }

    #[test]
    fn the_actor_is_excluded_so_editing_plus_ones_does_not_double_count() {
        let full_event_without_the_actor = context(Some(4), 3);
        assert_eq!(
            full_event_without_the_actor.evaluate(0),
            Ok(Admission::Admitted)
        );
    }

    #[test]
    fn plus_ones_beyond_the_event_limit_are_rejected() {
        assert_eq!(
            context(Some(100), 0).evaluate(5),
            Err(DomainError::PlusOnesExceeded {
                requested: 5,
                max: 2
            })
        );
    }

    #[test]
    fn negative_plus_ones_are_rejected() {
        assert_eq!(
            context(Some(100), 0).evaluate(-1),
            Err(DomainError::PlusOnesNegative)
        );
    }

    #[test]
    fn remaining_seats_never_reports_a_negative() {
        assert_eq!(context(Some(10), 12).remaining_seats(), Some(0));
        assert_eq!(context(None, 12).remaining_seats(), None);
    }
}
