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

