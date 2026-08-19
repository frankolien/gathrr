use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("a record that must exist was not found")]
    NotFound,

    #[error("{constraint} already holds this value")]
    Conflict { constraint: String },

    #[error("stored {column} value {value} is not a known variant")]
    UnknownVariant { column: &'static str, value: String },

    #[error(transparent)]
    Backend(#[from] sqlx::Error),
}

