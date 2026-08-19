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

#[derive(Serialize)]
struct Outgoing<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    text: &'a str,
}

#[derive(Deserialize)]
struct Refusal {
    message: String,
}

impl Resend {
    pub fn new(api_key: String, from: String) -> Option<Self> {
        let configured = !api_key.trim().is_empty() && !from.trim().is_empty();

        configured.then(|| Self {
            api_key,
            from,
            client: reqwest::Client::new(),
        })
    }

    pub async fn send(&self, to: &str, subject: &str, text: &str) -> Result<(), EmailError> {
        let response = self
            .client
            .post(ENDPOINT)
            .bearer_auth(&self.api_key)
            .json(&Outgoing {
                from: &self.from,
                to: [to],
                subject,
                text,
            })
            .send()
            .await
            .map_err(|error| EmailError::Unreachable(error.to_string()))?;

        if response.status().is_success() {
            return Ok(());
        }

        let refusal = response
            .json::<Refusal>()
            .await
            .map(|body| body.message)
            .unwrap_or_else(|_| "no reason given".to_owned());

        Err(EmailError::Refused(refusal))
    }
}

