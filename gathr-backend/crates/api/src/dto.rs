use gathr_application::events::EventDetail;
use gathr_application::rsvps::RsvpView;
use gathr_domain::{Category, EventStatus, RsvpStatus};
use gathr_infra_db::notifications::NotificationRecord;
use gathr_infra_db::{EventSummaryRecord, GuestRecord, InviteRecord};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct DevSignInRequest {
    pub display_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthSignInRequest {
    pub provider: String,
    pub id_token: String,
    pub nonce: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OtpRequest {
    pub channel: String,
    pub destination: String,
}

#[derive(Debug, Serialize)]
pub struct OtpChallengeResponse {
    pub destination: String,
    pub expires_in_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub development_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OtpVerifyRequest {
    pub channel: String,
    pub destination: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub apns_token: String,
    pub environment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct MuteRequest {
    pub muted: bool,
}

#[derive(Debug, Deserialize)]
pub struct EditEventRequest {
    pub title: Option<String>,
    pub category: Option<Category>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub location_name: Option<Option<String>>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(default, deserialize_with = "double_option_time")]
    pub ends_at: Option<Option<OffsetDateTime>>,
    pub timezone: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub capacity: Option<Option<i32>>,
    pub max_plus_ones: Option<i32>,
}

fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer).map(Some)
}

fn double_option_time<'de, D>(deserializer: D) -> Result<Option<Option<OffsetDateTime>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "time::serde::rfc3339::option")] Option<OffsetDateTime>);

    Wrapper::deserialize(deserializer).map(|wrapper| Some(wrapper.0))
}

#[derive(Debug, Serialize)]
pub struct UploadTicketResponse {
    pub upload_url: String,
    pub api_key: String,
    pub folder: String,
    pub timestamp: i64,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMediaRequest {
    pub public_id: String,
    pub content_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct RecordMediaResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PostMessageRequest {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub sender_id: Uuid,
    pub sender_display_name: String,
    pub seq: i64,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<gathr_application::messages::MessageView> for MessageResponse {
    fn from(view: gathr_application::messages::MessageView) -> Self {
        Self {
            id: view.id,
            event_id: view.event_id,
            sender_id: view.sender_id,
            sender_display_name: view.sender_display_name,
            seq: view.seq,
            body: view.body,
            created_at: view.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub latest_seq: i64,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub bio: Option<Option<String>>,
    pub avatar_media_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub display_name: String,
    pub is_guest: bool,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEventRequest {
    pub title: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub location_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub starts_at: OffsetDateTime,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub capacity: Option<i32>,
    #[serde(default)]
    pub max_plus_ones: Option<i32>,
    #[serde(default)]
    pub publish_now: bool,
}

#[derive(Debug, Serialize)]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub category: Category,
    pub location_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    pub timezone: String,
    pub status: EventStatus,
    pub capacity: Option<i32>,
    pub going_guests: i32,
    pub preview_guest_names: Vec<String>,
}

