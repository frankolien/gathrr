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

