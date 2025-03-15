use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod challenge;
mod resolver;
mod verification;

pub use challenge::{AuthChallenge, ChallengeConfig};
pub use resolver::{DIDResolver, MultiResolver};
pub use verification::{DIDVerifier, SignatureProof};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDIDInfo {
    pub user_id: Uuid,
    pub did: String,
    pub verification_methods: Vec<VerificationMethod>,
    pub created_at: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub type_: String,
    pub controller: String,
    pub public_key_base58: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DIDError {
    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),
    
    #[error("DID resolution failed: {0}")]
    ResolutionFailed(String),
    
    #[error("Verification method not found: {0}")]
    VerificationMethodNotFound(String),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Challenge expired")]
    ChallengeExpired,
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub type Result<T> = std::result::Result<T, DIDError>; 