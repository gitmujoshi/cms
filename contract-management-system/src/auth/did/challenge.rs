use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::did::{DIDError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub challenge_id: String,
    pub did: String,
    pub nonce: String,
    pub timestamp: DateTime<Utc>,
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ChallengeConfig {
    pub timeout_seconds: u64,
    pub nonce_bytes: usize,
}

impl Default for ChallengeConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 300, // 5 minutes
            nonce_bytes: 32,
        }
    }
}

impl AuthChallenge {
    pub fn new(did: String) -> Self {
        Self::new_with_config(did, &ChallengeConfig::default())
    }

    pub fn new_with_config(did: String, config: &ChallengeConfig) -> Self {
        let now = Utc::now();
        Self {
            challenge_id: Uuid::new_v4().to_string(),
            did,
            nonce: generate_nonce(config.nonce_bytes),
            timestamp: now,
            expiration: now + Duration::seconds(config.timeout_seconds as i64),
        }
    }

    pub fn verify_expiration(&self) -> Result<()> {
        if Utc::now() > self.expiration {
            return Err(DIDError::ChallengeExpired);
        }
        Ok(())
    }

    pub fn generate_message(&self) -> String {
        format!(
            "Sign this challenge for DID authentication:\nChallenge ID: {}\nDID: {}\nNonce: {}\nTimestamp: {}",
            self.challenge_id, self.did, self.nonce, self.timestamp
        )
    }
}

fn generate_nonce(bytes: usize) -> String {
    let mut rng = thread_rng();
    let mut bytes = vec![0u8; bytes];
    rng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_creation() {
        let did = "did:example:123".to_string();
        let challenge = AuthChallenge::new(did.clone());

        assert_eq!(challenge.did, did);
        assert_eq!(challenge.nonce.len(), 64); // 32 bytes = 64 hex chars
        assert!(challenge.expiration > challenge.timestamp);
    }

    #[test]
    fn test_challenge_expiration() {
        let config = ChallengeConfig {
            timeout_seconds: 0,
            nonce_bytes: 32,
        };
        let challenge = AuthChallenge::new_with_config("did:example:123".to_string(), &config);

        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(challenge.verify_expiration().is_err());
    }

    #[test]
    fn test_challenge_message_format() {
        let did = "did:example:123".to_string();
        let challenge = AuthChallenge::new(did.clone());
        let message = challenge.generate_message();

        assert!(message.contains(&challenge.challenge_id));
        assert!(message.contains(&did));
        assert!(message.contains(&challenge.nonce));
    }
} 