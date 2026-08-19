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

impl DbError {
    pub fn from_sqlx(error: sqlx::Error) -> Self {
        match &error {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
                Self::Conflict {
                    constraint: database_error
                        .constraint()
                        .unwrap_or("a unique index")
                        .to_owned(),
                }
            }
            _ => Self::Backend(error),
        }
    }

    pub fn is_conflict(&self) -> bool {
        matches!(self, Self::Conflict { .. })
    }
}
