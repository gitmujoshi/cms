use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::did::{DIDError, MultiResolver, Result, VerificationMethod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureProof {
    pub type_: String,
    pub created: DateTime<Utc>,
    pub verification_method: String,
    pub proof_purpose: String,
    pub proof_value: String,
}

pub struct DIDVerifier {
    resolver: Arc<MultiResolver>,
}

impl DIDVerifier {
    pub fn new(resolver: Arc<MultiResolver>) -> Self {
        Self { resolver }
    }

    pub async fn verify_signature(
        &self,
        did: &str,
        message: &str,
        signature: &str,
        verification_method_id: &str,
    ) -> Result<bool> {
        // Resolve DID document
        let document = self.resolver.resolve(did).await?;

        // Find verification method
        let method = document
            .verification_method
            .iter()
            .find(|vm| vm.id == verification_method_id)
            .ok_or_else(|| DIDError::VerificationMethodNotFound(verification_method_id.to_string()))?;

        // Verify signature using appropriate method
        self.verify_with_method(message, signature, method).await
    }

    async fn verify_with_method(
        &self,
        message: &str,
        signature: &str,
        method: &VerificationMethod,
    ) -> Result<bool> {
        match method.type_.as_str() {
            "Ed25519VerificationKey2018" => verify_ed25519(message, signature, &method.public_key_base58),
            "EcdsaSecp256k1VerificationKey2019" => verify_secp256k1(message, signature, &method.public_key_base58),
            _ => Err(DIDError::ResolutionFailed(format!(
                "Unsupported verification method type: {}",
                method.type_
            ))),
        }
    }
}

fn verify_ed25519(message: &str, signature: &str, public_key: &str) -> Result<bool> {
    // Decode public key and signature from base58
    let public_key = bs58::decode(public_key)
        .into_vec()
        .map_err(|e| DIDError::InvalidSignature)?;
    let signature = bs58::decode(signature)
        .into_vec()
        .map_err(|e| DIDError::InvalidSignature)?;

    // Verify using ed25519-dalek
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key)
        .map_err(|_| DIDError::InvalidSignature)?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature)
        .map_err(|_| DIDError::InvalidSignature)?;

    Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
}

fn verify_secp256k1(message: &str, signature: &str, public_key: &str) -> Result<bool> {
    // Decode public key and signature
    let public_key = bs58::decode(public_key)
        .into_vec()
        .map_err(|_| DIDError::InvalidSignature)?;
    let signature = hex::decode(signature)
        .map_err(|_| DIDError::InvalidSignature)?;

    // Hash message (keccak256)
    let message = keccak_hash::keccak256(message.as_bytes());

    // Verify using secp256k1
    let secp = secp256k1::Secp256k1::verification_only();
    let public_key = secp256k1::PublicKey::from_slice(&public_key)
        .map_err(|_| DIDError::InvalidSignature)?;
    let signature = secp256k1::Signature::from_compact(&signature)
        .map_err(|_| DIDError::InvalidSignature)?;
    let message = secp256k1::Message::from_slice(&message)
        .map_err(|_| DIDError::InvalidSignature)?;

    Ok(secp.verify(&message, &signature, &public_key).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;

    #[test]
    fn test_ed25519_verification() {
        let mut csprng = OsRng;
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let message = b"Test message";
        let signature = keypair.sign(message);

        let public_key_base58 = bs58::encode(keypair.public.as_bytes()).into_string();
        let signature_base58 = bs58::encode(signature.to_bytes()).into_string();

        let result = verify_ed25519(
            std::str::from_utf8(message).unwrap(),
            &signature_base58,
            &public_key_base58,
        );

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_invalid_ed25519_signature() {
        let mut csprng = OsRng;
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let public_key_base58 = bs58::encode(keypair.public.as_bytes()).into_string();

        let result = verify_ed25519(
            "Test message",
            "invalid_signature",
            &public_key_base58,
        );

        assert!(result.is_err());
    }
} 