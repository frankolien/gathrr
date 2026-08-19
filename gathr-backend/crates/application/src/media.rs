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

