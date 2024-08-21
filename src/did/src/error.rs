use candid::CandidType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, UpgraderError>;

#[derive(Debug, Error, Deserialize, CandidType, Eq, PartialEq, Serialize, Clone)]
pub enum UpgraderError {
    #[error("the user has no permission to call this method")]
    NotAuthorized,

    #[error("Anonymous principal is not allowed")]
    AnonymousPrincipalNotAllowed,

    #[error("The request is not valid: {0}")]
    BadRequest(String),

    #[error("The key provided already exists: {0}")]
    NotUniqueKey(String),
}
