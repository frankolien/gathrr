use serde::{Deserialize, Serialize};
use thiserror::Error;

const ENDPOINT: &str = "https://api.resend.com/emails";

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("the mail provider could not be reached: {0}")]
    Unreachable(String),
    #[error("the mail provider refused this message: {0}")]
    Refused(String),
}

#[derive(Debug, Clone)]
pub struct Resend {
    api_key: String,
    from: String,
    client: reqwest::Client,
}

