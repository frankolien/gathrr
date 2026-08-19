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

