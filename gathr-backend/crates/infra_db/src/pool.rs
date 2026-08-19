use sqlx::postgres::{PgPoolOptions, Postgres};
use sqlx::{Pool, Transaction};

use crate::error::DbError;

pub type Db = Pool<Postgres>;
pub type Tx<'a> = Transaction<'a, Postgres>;

pub async fn connect(database_url: &str, max_connections: u32) -> Result<Db, DbError> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(DbError::from_sqlx)
}

pub async fn run_migrations(db: &Db) -> Result<(), DbError> {
    sqlx::migrate!("../../migrations")
        .run(db)
        .await
        .map_err(|error| DbError::Backend(sqlx::Error::Migrate(Box::new(error))))
}
