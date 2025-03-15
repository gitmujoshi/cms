# DID Integration for Authentication and Contract Signing

## Overview

This guide explains how to integrate Decentralized Identity (DID) for both user authentication and digital contract signing in the Contract Management System.

## Table of Contents
1. [Authentication Integration](#authentication-integration)
2. [Contract Signing Integration](#contract-signing-integration)
3. [Implementation Details](#implementation-details)
4. [Security Considerations](#security-considerations)

## Authentication Integration

### 1. User DID Registration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDIDInfo {
    pub user_id: Uuid,
    pub did: String,
    pub verification_methods: Vec<VerificationMethod>,
    pub created_at: DateTime<Utc>,
    pub last_verified: DateTime<Utc>,
}

impl User {
    pub async fn register_did(&mut self, did: String) -> Result<UserDIDInfo, Error> {
        // 1. Verify DID ownership through challenge-response
        let challenge = AuthChallenge::new(did.clone());
        // ... challenge verification process ...

        // 2. Resolve DID document to get verification methods
        let document = self.resolver.resolve(&did).await?;
        
        // 3. Store DID information
        let did_info = UserDIDInfo {
            user_id: self.id,
            did,
            verification_methods: document.verification_method,
            created_at: Utc::now(),
            last_verified: Utc::now(),
        };
        
        self.store_did_info(&did_info).await?;
        Ok(did_info)
    }
}
```

### 2. DID-based Authentication

```rust
pub struct DIDAuthenticator {
    resolver: Arc<MultiResolver>,
    store: Arc<RedisStore>,
}

impl DIDAuthenticator {
    pub async fn authenticate(&self, did: &str, signature: &str) -> Result<AuthToken, Error> {
        // 1. Verify DID ownership
        let user = self.verify_did_ownership(did, signature).await?;
        
        // 2. Generate session token
        let claims = JWTClaims {
            sub: user.id.to_string(),
            did: did.to_string(),
            role: user.role,
            exp: Utc::now() + Duration::hours(24),
        };
        
        let token = generate_jwt_token(&claims)?;
        Ok(AuthToken { token })
    }
}
```

## Contract Signing Integration

### 1. Contract Signature Structure

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractSignature {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub signer_did: String,
    pub signature: String,
    pub signature_type: SignatureType,
    pub verification_method: String,
    pub signed_at: DateTime<Utc>,
    pub proof: SignatureProof,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureProof {
    pub type_: String,
    pub created: DateTime<Utc>,
    pub verification_method: String,
    pub proof_purpose: String,
    pub proof_value: String,
}
```

### 2. Contract Signing Process

```rust
impl Contract {
    pub async fn sign_with_did(
        &mut self,
        signer_did: &str,
        signature: &str,
        verification_method: &str,
    ) -> Result<ContractSignature, Error> {
        // 1. Verify signer's DID and authorization
        self.verify_signer_authorization(signer_did).await?;
        
        // 2. Create signature record
        let sig = ContractSignature {
            id: Uuid::new_v4(),
            contract_id: self.id,
            signer_did: signer_did.to_string(),
            signature: signature.to_string(),
            signature_type: SignatureType::Ed25519,
            verification_method: verification_method.to_string(),
            signed_at: Utc::now(),
            proof: self.generate_signature_proof(signer_did, signature).await?,
        };
        
        // 3. Update contract status
        self.update_status_after_signature(&sig).await?;
        
        // 4. Store signature
        self.store_signature(&sig).await?;
        
        Ok(sig)
    }

    async fn verify_signer_authorization(&self, signer_did: &str) -> Result<(), Error> {
        // Check if signer is authorized (provider or consumer)
        if signer_did != self.provider_did && signer_did != self.consumer_did {
            return Err(Error::UnauthorizedSigner);
        }
        
        // Check if signature is already present
        if self.signatures.iter().any(|s| s.signer_did == signer_did) {
            return Err(Error::AlreadySigned);
        }
        
        Ok(())
    }

    async fn generate_signature_proof(
        &self,
        signer_did: &str,
        signature: &str,
    ) -> Result<SignatureProof, Error> {
        // Create verifiable proof of signature
        let proof = SignatureProof {
            type_: "Ed25519Signature2020".to_string(),
            created: Utc::now(),
            verification_method: format!("{}#keys-1", signer_did),
            proof_purpose: "contractSigning".to_string(),
            proof_value: signature.to_string(),
        };
        
        Ok(proof)
    }
}
```

### 3. Signature Verification

```rust
pub struct SignatureVerifier {
    resolver: Arc<MultiResolver>,
}

impl SignatureVerifier {
    pub async fn verify_signature(
        &self,
        contract: &Contract,
        signature: &ContractSignature,
    ) -> Result<bool, Error> {
        // 1. Resolve signer's DID document
        let did_doc = self.resolver.resolve(&signature.signer_did).await?;
        
        // 2. Get verification method
        let verification_method = did_doc
            .verification_method
            .iter()
            .find(|vm| vm.id == signature.verification_method)
            .ok_or(Error::VerificationMethodNotFound)?;
        
        // 3. Verify signature
        let message = contract.generate_signing_message()?;
        self.verify_signature_with_method(
            &message,
            &signature.signature,
            verification_method,
        ).await
    }
}
```

## Implementation Details

### 1. Contract Status Management

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ContractStatus {
    Draft,
    PendingSignatures,
    Active,
    Suspended,
    Terminated,
}

impl Contract {
    async fn update_status_after_signature(
        &mut self,
        signature: &ContractSignature,
    ) -> Result<(), Error> {
        // Count valid signatures
        let valid_signatures = self.signatures.len() + 1;
        
        // Update status based on signature count
        self.status = match valid_signatures {
            1 => ContractStatus::PendingSignatures,
            2 => ContractStatus::Active, // Both parties have signed
            _ => self.status.clone(),
        };
        
        Ok(())
    }
}
```

### 2. DID Resolution Cache

```rust
pub struct DIDCache {
    cache: Arc<RwLock<HashMap<String, (Document, Instant)>>>,
    ttl: Duration,
}

impl DIDCache {
    pub async fn get_or_resolve(
        &self,
        did: &str,
        resolver: &MultiResolver,
    ) -> Result<Document, Error> {
        // Check cache first
        if let Some((doc, timestamp)) = self.cache.read().unwrap().get(did) {
            if timestamp.elapsed() < self.ttl {
                return Ok(doc.clone());
            }
        }
        
        // Resolve if not in cache or expired
        let document = resolver.resolve(did).await?;
        self.cache.write().unwrap().insert(
            did.to_string(),
            (document.clone(), Instant::now()),
        );
        
        Ok(document)
    }
}
```

## Security Considerations

### 1. Signature Validation

```rust
impl Contract {
    fn validate_signature_requirements(&self) -> Result<(), Error> {
        // 1. Check contract status
        if self.status != ContractStatus::Draft && 
           self.status != ContractStatus::PendingSignatures {
            return Err(Error::InvalidContractStatus);
        }
        
        // 2. Verify signature timestamp
        if self.valid_from > Utc::now() {
            return Err(Error::ContractNotYetValid);
        }
        
        // 3. Check signature order (if required)
        if let Some(required_order) = &self.signature_order {
            self.validate_signature_order(required_order)?;
        }
        
        Ok(())
    }
}
```

### 2. DID Verification

```rust
impl DIDVerifier {
    pub async fn verify_did_control(
        &self,
        did: &str,
        proof: &SignatureProof,
    ) -> Result<bool, Error> {
        // 1. Resolve DID document
        let document = self.resolver.resolve(did).await?;
        
        // 2. Verify proof against verification methods
        let valid = document
            .verification_method
            .iter()
            .any(|vm| self.verify_proof_with_method(proof, vm).is_ok());
        
        // 3. Check for revocation
        if valid {
            self.check_revocation_status(did, proof).await?;
        }
        
        Ok(valid)
    }
}
```

### 3. Audit Logging

```rust
pub async fn log_signature_event(
    contract_id: Uuid,
    signer_did: &str,
    event_type: SignatureEventType,
    result: Result<(), Error>,
) {
    let event = SignatureEvent {
        contract_id,
        signer_did: signer_did.to_string(),
        event_type,
        timestamp: Utc::now(),
        success: result.is_ok(),
        error: result.err().map(|e| e.to_string()),
    };
    
    tracing::info!(
        contract_id = %contract_id,
        signer = %signer_did,
        event = ?event_type,
        success = event.success,
        error = ?event.error,
        "Contract signature event"
    );
}
```

## Usage Example

```rust
// 1. User authentication with DID
let auth_token = authenticator
    .authenticate("did:example:123", signature)
    .await?;

// 2. Create contract
let mut contract = Contract::new(
    provider_did: "did:example:123",
    consumer_did: "did:example:456",
    // ... other contract details ...
);

// 3. Sign contract with DID
let signature = contract
    .sign_with_did(
        "did:example:123",
        signature,
        "did:example:123#keys-1",
    )
    .await?;

// 4. Verify signature
let is_valid = verifier
    .verify_signature(&contract, &signature)
    .await?;
```

This integration provides:
- Secure DID-based authentication
- Verifiable digital signatures for contracts
- Proof of signing authority
- Audit trail for all signature operations
- Caching for improved performance
- Comprehensive error handling 