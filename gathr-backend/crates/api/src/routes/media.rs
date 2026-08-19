use actix_web::{web, HttpResponse};
use gathr_application::media;
use gathr_infra_media::{AVATAR_FOLDER, COVER_FOLDER};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::{RecordMediaRequest, RecordMediaResponse, UploadTicketResponse};
use crate::error::ApiError;
use crate::extract::AuthUser;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct TicketRequest {
    pub purpose: Option<String>,
}

pub async fn sign(
    state: web::Data<AppState>,
    _user: AuthUser,
    body: web::Json<TicketRequest>,
) -> Result<HttpResponse, ApiError> {
    let folder = match body.purpose.as_deref() {
        Some("avatar") => AVATAR_FOLDER,
        None | Some("cover") => COVER_FOLDER,
        Some(other) => {
            return Err(ApiError(gathr_application::AppError::Validation(format!(
                "{other} is not something this app uploads"
            ))))
        }
    };

    let ticket = media::ticket(state.cloudinary.as_ref(), folder, OffsetDateTime::now_utc())?;

    Ok(HttpResponse::Ok().json(UploadTicketResponse {
        upload_url: ticket.upload_url,
        api_key: ticket.api_key,
        folder: ticket.folder,
        timestamp: ticket.timestamp,
        signature: ticket.signature,
    }))
}

