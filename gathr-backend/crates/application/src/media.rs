use time::OffsetDateTime;
use uuid::Uuid;

use gathr_infra_db::{media, Db};
use gathr_infra_media::{Cloudinary, UploadTicket};

use crate::error::AppError;

pub const COVER_TRANSFORMATION: &str = "c_fill,w_1200,h_800,q_auto,f_auto";
pub const AVATAR_TRANSFORMATION: &str = "c_fill,g_face,w_256,h_256,q_auto,f_auto";

pub fn ticket(
    cloudinary: Option<&Cloudinary>,
    folder: &str,
    now: OffsetDateTime,
) -> Result<UploadTicket, AppError> {
    cloudinary
        .ok_or(AppError::ProviderUnavailable)?
        .ticket(folder, now.unix_timestamp())
        .map_err(|error| AppError::Validation(error.to_string()))
}

pub async fn record(
    db: &Db,
    owner_id: Uuid,
    public_id: &str,
    content_type: &str,
    width: Option<i32>,
    height: Option<i32>,
) -> Result<Uuid, AppError> {
    let public_id = public_id.trim();
    if public_id.is_empty() {
        return Err(AppError::Validation(
            "the upload did not return an identifier".to_owned(),
        ));
    }
    if !content_type.starts_with("image/") {
        return Err(AppError::Validation(
            "only images can be attached".to_owned(),
        ));
    }

    let stored = media::insert(db, owner_id, public_id, content_type, width, height).await?;
    Ok(stored.id)
}

pub async fn attach_cover(
    db: &Db,
    event_id: Uuid,
    owner_id: Uuid,
    media_id: Uuid,
) -> Result<(), AppError> {
    media::find_owned(db, media_id, owner_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(media::attach_cover(db, event_id, Some(media_id)).await?)
}

pub async fn cover_url(
    db: &Db,
    cloudinary: Option<&Cloudinary>,
    event_id: Uuid,
) -> Result<Option<String>, AppError> {
    let (Some(cloudinary), Some(key)) =
        (cloudinary, media::bucket_key_for_event(db, event_id).await?)
    else {
        return Ok(None);
    };

    Ok(Some(cloudinary.delivery_url(&key, COVER_TRANSFORMATION)))
}

pub async fn avatar_url(
    db: &Db,
    cloudinary: Option<&Cloudinary>,
    user_id: Uuid,
) -> Result<Option<String>, AppError> {
    let (Some(cloudinary), Some(key)) =
        (cloudinary, media::bucket_key_for_user(db, user_id).await?)
    else {
        return Ok(None);
    };

    Ok(Some(cloudinary.delivery_url(&key, AVATAR_TRANSFORMATION)))
}
