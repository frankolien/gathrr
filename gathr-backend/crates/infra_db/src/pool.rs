use sqlx::postgres::{PgPoolOptions, Postgres};
use sqlx::{Pool, Transaction};

use crate::error::DbError;

pub type Db = Pool<Postgres>;
pub type Tx<'a> = Transaction<'a, Postgres>;

