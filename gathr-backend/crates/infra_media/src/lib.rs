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

#[cfg(test)]
mod tests {
    use super::*;

    fn cloudinary() -> Cloudinary {
        Cloudinary::new("demo".to_owned(), "key".to_owned(), "secret".to_owned()).unwrap()
    }

    #[test]
    fn a_missing_value_leaves_the_uploader_unconfigured_rather_than_half_built() {
        assert!(Cloudinary::new(String::new(), "key".to_owned(), "secret".to_owned()).is_none());
        assert!(Cloudinary::new("demo".to_owned(), "  ".to_owned(), "secret".to_owned()).is_none());
        assert!(cloudinary().ticket(COVER_FOLDER, 1).is_ok());
    }

    #[test]
    fn the_signature_is_sha1_of_the_sorted_params_followed_by_the_secret() {
        let ticket = cloudinary().ticket(COVER_FOLDER, 1_700_000_000).unwrap();

        let mut expected = Sha1::new();
        expected.update(b"folder=gathr/covers&timestamp=1700000000");
        expected.update(b"secret");
        let expected: String = expected
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        assert_eq!(ticket.signature, expected);
        assert_eq!(ticket.signature.len(), 40);
    }

    #[test]
    fn a_different_timestamp_produces_a_different_signature() {
        let first = cloudinary().ticket(COVER_FOLDER, 1_700_000_000).unwrap();
        let second = cloudinary().ticket(COVER_FOLDER, 1_700_000_001).unwrap();
        assert_ne!(first.signature, second.signature);
    }

    #[test]
    fn uploads_are_confined_to_folders_this_app_owns() {
        assert!(matches!(
            cloudinary().ticket("../../etc", 1),
            Err(MediaError::UnknownFolder(_))
        ));
        assert!(cloudinary().ticket(AVATAR_FOLDER, 1).is_ok());
    }

    #[test]
    fn the_secret_never_appears_in_the_ticket_handed_to_a_client() {
        let ticket = cloudinary().ticket(COVER_FOLDER, 1_700_000_000).unwrap();
        let rendered = format!("{ticket:?}");
        assert!(
            !rendered.contains("secret"),
            "a ticket is sent to the phone and must never carry the api secret"
        );
    }
}
