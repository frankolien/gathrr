use serde_json::Value;
use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

#[derive(Debug, Clone)]
pub struct StoredResponse {
    pub request_hash: String,
    pub response_code: Option<i32>,
    pub response_body: Option<Value>,
}

pub async fn find(db: &Db, key: &str, user_id: Uuid) -> Result<Option<StoredResponse>, DbError> {
    sqlx::query_as!(
        StoredResponse,
        r#"SELECT request_hash, response_code, response_body
           FROM idempotency_keys
           WHERE key = $1 AND user_id = $2 AND created_at > now() - interval '24 hours'"#,
        key,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

