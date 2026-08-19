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

impl From<EventSummaryRecord> for EventSummary {
    fn from(record: EventSummaryRecord) -> Self {
        Self {
            id: record.event.id,
            title: record.event.title,
            category: record.event.category,
            location_name: record.event.location_name,
            starts_at: record.event.starts_at,
            ends_at: record.event.ends_at,
            timezone: record.event.timezone,
            status: record.event.status,
            capacity: record.event.capacity,
            going_guests: record.going_guests,
            preview_guest_names: record.preview_guest_names,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventDetailResponse {
    #[serde(flatten)]
    pub summary: EventSummary,
    pub description: Option<String>,
    pub host_display_name: String,
    pub observed_status: EventStatus,
    pub going_guests: i32,
    pub max_plus_ones: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub server_time: OffsetDateTime,
}

impl From<EventDetail> for EventDetailResponse {
    fn from(detail: EventDetail) -> Self {
        let description = detail.event.description.clone();
        let max_plus_ones = detail.event.max_plus_ones;
        let going_guests = detail.going_guests;
        Self {
            summary: EventSummary::from(EventSummaryRecord {
                event: detail.event,
                going_guests,
                preview_guest_names: detail.preview_guest_names,
            }),
            description,
            host_display_name: detail.host_display_name,
            observed_status: detail.observed_status,
            going_guests: detail.going_guests,
            max_plus_ones,
            server_time: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_filter")]
    pub filter: String,
}

fn default_filter() -> String {
    "this_week".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    #[serde(default)]
    pub max_uses: Option<i32>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub code: String,
    pub url: String,
    pub max_uses: Option<i32>,
    pub uses: i32,
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
}

impl InviteResponse {
    pub fn new(record: InviteRecord, base_url: &str) -> Self {
        Self {
            url: format!("{base_url}/i/{}", record.code),
            id: record.id,
            event_id: record.event_id,
            code: record.code,
            max_uses: record.max_uses,
            uses: record.uses,
            expires_at: record.expires_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PublicInviteResponse {
    pub event_id: Uuid,
    pub title: String,
    pub category: Category,
    pub location_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub starts_at: OffsetDateTime,
    pub timezone: String,
    pub host_first_name: String,
    pub going_guests: i32,
}

#[derive(Debug, Deserialize)]
pub struct RsvpRequestBody {
    pub status: RsvpStatus,
    #[serde(default)]
    pub plus_ones: i32,
    #[serde(default)]
    pub accept_waitlist: bool,
}

#[derive(Debug, Serialize)]
pub struct RsvpResponse {
    pub event_id: Uuid,
    pub status: RsvpStatus,
    pub plus_ones: i32,
    pub entered_waitlist: bool,
    pub seats_remaining: Option<i32>,
}

impl From<RsvpView> for RsvpResponse {
    fn from(view: RsvpView) -> Self {
        Self {
            event_id: view.event_id,
            status: view.status,
            plus_ones: view.plus_ones,
            entered_waitlist: view.entered_waitlist,
            seats_remaining: view.seats_remaining,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GuestResponse {
    pub user_id: Uuid,
    pub display_name: String,
    pub status: RsvpStatus,
    pub plus_ones: i32,
}

impl From<GuestRecord> for GuestResponse {
    fn from(record: GuestRecord) -> Self {
        Self {
            user_id: record.user_id,
            display_name: record.display_name,
            status: record.status,
            plus_ones: record.plus_ones,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GuestListResponse {
    pub going: i32,
    pub seats_taken: i32,
    pub guests: Vec<GuestResponse>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationFeedQuery {
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub before: Option<OffsetDateTime>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MarkReadRequest {
    #[serde(default)]
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub kind: String,
    pub event_id: Uuid,
    pub event_title: String,
    pub actor_display_name: Option<String>,
    pub read: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<NotificationRecord> for NotificationResponse {
    fn from(record: NotificationRecord) -> Self {
        Self {
            id: record.id,
            kind: record.kind,
            event_id: record.event_id,
            event_title: record.event_title,
            actor_display_name: record.actor_display_name,
            read: record.read_at.is_some(),
            created_at: record.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationFeedResponse {
    pub unread: i64,
    pub notifications: Vec<NotificationResponse>,
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub unread: i64,
}
