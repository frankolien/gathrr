use uuid::Uuid;

use crate::error::DbError;
use crate::pool::Db;

#[derive(Debug, Clone)]
pub struct MediaRecord {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub bucket_key: String,
    pub content_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

pub async fn insert(
    db: &Db,
    owner_id: Uuid,
    bucket_key: &str,
    content_type: &str,
    width: Option<i32>,
    height: Option<i32>,
) -> Result<MediaRecord, DbError> {
    sqlx::query_as!(
        MediaRecord,
        r#"INSERT INTO media (owner_id, bucket_key, content_type, width, height)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, owner_id, bucket_key, content_type, width, height"#,
        owner_id,
        bucket_key,
        content_type,
        width,
        height
    )
    .fetch_one(db)
    .await
    .map_err(DbError::from_sqlx)
}

pub async fn find_owned(db: &Db, id: Uuid, owner_id: Uuid) -> Result<Option<MediaRecord>, DbError> {
    sqlx::query_as!(
        MediaRecord,
        r#"SELECT id, owner_id, bucket_key, content_type, width, height
           FROM media WHERE id = $1 AND owner_id = $2"#,
        id,
        owner_id
    )
    .fetch_optional(db)
    .await
    .map_err(DbError::from_sqlx)
}

