use sha1::{Digest, Sha1};
use thiserror::Error;

pub const COVER_FOLDER: &str = "gathr/covers";
pub const AVATAR_FOLDER: &str = "gathr/avatars";

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("image hosting is not configured")]
    NotConfigured,
    #[error("{0} is not a folder this app uploads to")]
    UnknownFolder(String),
}

#[derive(Debug, Clone)]
pub struct Cloudinary {
    cloud_name: String,
    api_key: String,
    api_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadTicket {
    pub upload_url: String,
    pub api_key: String,
    pub folder: String,
    pub timestamp: i64,
    pub signature: String,
}

impl Cloudinary {
    pub fn new(cloud_name: String, api_key: String, api_secret: String) -> Option<Self> {
        let configured = [&cloud_name, &api_key, &api_secret]
            .iter()
            .all(|value| !value.trim().is_empty());

        configured.then_some(Self {
            cloud_name,
            api_key,
            api_secret,
        })
    }

    pub fn ticket(&self, folder: &str, timestamp: i64) -> Result<UploadTicket, MediaError> {
        if folder != COVER_FOLDER && folder != AVATAR_FOLDER {
            return Err(MediaError::UnknownFolder(folder.to_owned()));
        }

        Ok(UploadTicket {
            upload_url: format!(
                "https://api.cloudinary.com/v1_1/{}/image/upload",
                self.cloud_name
            ),
            api_key: self.api_key.clone(),
            folder: folder.to_owned(),
            timestamp,
            signature: self.sign(&format!("folder={folder}&timestamp={timestamp}")),
        })
    }

    pub fn delivery_url(&self, public_id: &str, transformation: &str) -> String {
        format!(
            "https://res.cloudinary.com/{}/image/upload/{transformation}/{public_id}",
            self.cloud_name
        )
    }

    fn sign(&self, params: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(params.as_bytes());
        hasher.update(self.api_secret.as_bytes());
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

